use super::{Conditional, Executable, Instruction, Iterative, Rule, Variable, VariableMapType};
use crate::{
    capture::{
        create_capturer,
        pc_common::{get_network_ssid, Event, Window, KEYSTROKES, MOUSE_CLICKS},
    },
    graphql::{get_or_insert_user_ssid, send_user_event},
    rest_api::send_screenshots,
    scripting::ConditionalFn,
};

use regex::Regex;
use serde_json::Value;
use std::{convert::TryInto, sync::atomic::Ordering, time::Duration};

/// SAFETY: The use of a raw pointer (`*mut VariableMapType`) is safe as long as
/// you ensure that the value won't get dropped before the execution finishes,
/// which will be the case as long as you execute everything in a sequential order,
/// but if you try to do anything concurrently you should switch to
/// `Arc<ReadWrite<VariableMapType>>`, because that'll ensure there are no
/// data races and undefined behaviour in the case of mutable borrow of a the value.
/// Right now there's no chance of this causing undefined behaviour, because the
/// current execution model does not include concurrency and all executables are executed
/// one after another.

/// Returns the duration to wait for execution and a vector of executable pointers.
pub fn parse(
    string: impl AsRef<str>,
    variable_map: *mut VariableMapType,
) -> anyhow::Result<(Duration, Vec<Box<dyn Executable>>)> {
    let mut executables = vec![];

    let mut timeout: Option<Duration> = None;

    let string = string.as_ref().replace("\t", " ");

    let lines_split = string.split("\n");

    let mut lines: Vec<Vec<&str>> = vec![];

    for line in lines_split {
        if line.is_empty() {
            continue;
        }

        let word_positions = get_word_positions(line);

        let mut words: Vec<&str> = vec![];

        for (start, end) in word_positions {
            words.push(&line[start..end]);
        }

        lines.push(words);
    }

    let mut line_pos = 0;

    while line_pos < lines.len() {
        let line = &lines[line_pos];

        let word = line[0];

        match word {
            "EVERY" => {
                if line.get(1).is_none() {
                    anyhow::bail!(
                        "You haven't provided a number as second parameter at line {}",
                        line_pos
                    );
                } else if line.get(2).is_none() {
                    anyhow::bail!("You haven't provided a valid time variant at line {}.\nYour options are: MILLISECONDS, SECONDS, MINUTES and HOURS.", line_pos);
                }

                let amount: u64 = match line[1].parse() {
                    Ok(amount) => amount,
                    Err(_) => {
                        anyhow::bail!(
                            "You haven't provided a valid number as a second parameter at line {}.",
                            line_pos
                        )
                    }
                };

                let time_variant = line[2];

                let time = match time_variant {
                   "MILLISECONDS" => amount,
                   "SECONDS" => amount * 1000,
                   "MINUTES" => amount * 1000 * 60,
                   "HOURS" => amount * 1000 * 60 * 60,
                   _ => anyhow::bail!("The time variant {} you provided at line {} doesn't exist.\nYour options are: MILLISECONDS, SECONDS, MINUTES and HOURS.", time_variant, line_pos)
                };

                timeout = Some(Duration::from_millis(time));
            }
            "IF" => {
                executables.push(parse_conditional(&lines, &mut line_pos, variable_map)?.into());
            }
            "ITERATE" => {
                executables.push(parse_iterator(&lines, &mut line_pos, variable_map)?.into());
            }
            _ => {
                if let Some(instruction) = parse_instruction(&line, variable_map)? {
                    executables.push(instruction.into());
                }
            }
        };

        line_pos += 1;
    }

    match timeout {
        Some(timeout) => Ok((timeout, executables)),
        None => anyhow::bail!("You haven't specified the EVERY statement."),
    }
}

fn parse_instruction(
    line: &[&str],
    variable_map: *mut VariableMapType,
) -> anyhow::Result<Option<Instruction>> {
    let word = line[0];
    match word {
        "PRINT" => {
            debug!("Parsing PRINT Statement");
            let word = line[1];
            let is_string = is_string(word);

            let word = if is_string {
                word[1..word.len() - 1].to_owned()
            } else {
                word.to_owned()
            };

            let function: Instruction = if is_string {
                Box::new(move || {
                    println!("{}", word);
                    Ok(())
                })
                .into()
            } else {
                Box::new(move || {
                    let map = unsafe { &*variable_map };

                    let variable = match map.get(word.as_str()) {
                        Some(variable) => variable,
                        None => anyhow::bail!("Couldn't find the Variable with Key {}", word),
                    };

                    println!("{}", variable);

                    Ok(())
                })
                .into()
            };
            Ok(Some(function.into()))
        }
        "SAVE_TO_DB" => {
            let function = move || {
                debug!("Saving to DB");

                let map = unsafe { &mut *variable_map };

                let rule_id = match map.get("RULE_ID") {
                    Some(Variable::RcStr(string)) => (**string).clone(),
                    _ => anyhow::bail!("RULE_ID is not a String"),
                };

                let rule_body = match map.get("RULE_BODY") {
                    Some(Variable::RcStr(string)) => (**string).clone(),
                    _ => anyhow::bail!("RULE_BODY is not a String"),
                };

                let seconds_since_last_input = match map.get("SECONDS_SINCE_LAST_INPUT") {
                    Some(Variable::U64(int)) => *int,
                    Some(s) => {
                        dbg!(s);
                        anyhow::bail!("SECONDS_SINCE_LAST_INPUT is not a U64")
                    }
                    None => anyhow::bail!("SECONDS_SINCE_LAST_INPUT is not a U64"),
                };

                let windows: Vec<Window> = match map.get("WINDOWS") {
                    Some(Variable::Vector(vec)) => {
                        let mut windows = vec![];
                        for variable in vec.iter() {
                            let map = match variable {
                                Variable::Map(map) => map,
                                _ => anyhow::bail!("Variable is not a Map"),
                            };
                            windows.push((&**map).try_into()?);
                        }
                        windows
                    }
                    _ => anyhow::bail!("WINDOWS is not a Vector"),
                };

                let screenshots: Option<Box<Vec<Value>>> = match map.get("SCREENSHOTS") {
                    Some(Variable::SerdeJsonVector(value)) => Some(value.clone()),
                    _ => None,
                };

                let network: Option<String> = match map.get("NETWORK_SSID") {
                    Some(Variable::RcStr(string)) => {
                        match get_or_insert_user_ssid(string.as_str()) {
                            Ok(id) => Some(id),
                            Err(err) => {
                                error!("{}", err);
                                None
                            }
                        }
                    }
                    _ => None,
                };

                let event = Event {
                    windows,
                    rule: Some(Rule {
                        id: rule_id,
                        body: rule_body,
                    }),
                    network,
                    screenshots,
                    keyboard: KEYSTROKES.load(Ordering::Relaxed),
                    mouse: MOUSE_CLICKS.load(Ordering::Relaxed),
                    seconds_since_last_input,
                };

                KEYSTROKES.store(0, Ordering::SeqCst);
                MOUSE_CLICKS.store(0, Ordering::SeqCst);

                send_user_event(event)?;

                if let Some(Variable::SerdeJsonVector(screenshots)) = map.get_mut("SCREENSHOTS") {
                    // Clear the screenshots, so that we don't repeat them on the next save.
                    screenshots.clear();
                }

                Ok(())
            };
            Ok(Some(function.into()))
        }
        "GET_NETWORK_SSID" => {
            let function = move || {
                let map = unsafe { &mut *variable_map };
                if let Some(ssid) = get_network_ssid() {
                    map.insert("NETWORK_SSID", ssid.into());
                }
                Ok(())
            };
            Ok(Some(function.into()))
        }
        "GET_PERIPHERALS" => {
            let function = move || {
                let map = unsafe { &mut *variable_map };
                map.insert("KEYSTROKES", KEYSTROKES.load(Ordering::Relaxed).into());
                map.insert("MOUSE_CLICKS", MOUSE_CLICKS.load(Ordering::Relaxed).into());
                Ok(())
            };
            Ok(Some(function.into()))
        }
        "GET_WINDOWS" => {
            let mut capturer = create_capturer();
            let function = move || {
                let map = unsafe { &mut *variable_map };
                let event = capturer.capture()?;
                map.insert(
                    "WINDOWS",
                    event
                        .windows
                        .into_iter()
                        .map(|w| w.into())
                        .collect::<Vec<VariableMapType>>()
                        .into(),
                );
                map.insert(
                    "SECONDS_SINCE_LAST_INPUT",
                    event.seconds_since_last_input.into(),
                );
                Ok(())
            };
            Ok(Some(function.into()))
        }
        "CAPTURE_SCREEN" => {
            use captis::*;

            let variable = line
                .get(1)
                .map(|s| &s[1..s.len() - 1])
                .ok_or_else(|| anyhow!("You haven't provided the screen number to capture"))?;

            let capturer =
                init_capturer().or_else(|_| Err(anyhow!("Couldn't initiate Screen capturer")))?;

            return match variable {
                "ALL" => {
                    unsafe {
                        (&mut *variable_map)
                            .insert("SCREENSHOTS", Variable::SerdeJsonVector(Box::new(vec![])));
                    }
                    let function = move || {
                        let map = unsafe { &mut *variable_map };

                        let images = capturer.capture_all()?;

                        let mut file_vec = send_screenshots(&images)?;

                        let vec = match map
                            .get_mut("SCREENSHOTS")
                            .ok_or_else(|| anyhow!("Couldn't get SCREENSHOTS from VariableMap"))?
                        {
                            Variable::SerdeJsonVector(vec) => vec,
                            _ => anyhow::bail!("Variable is not a SerdeJsonVector"),
                        };

                        vec.append(&mut file_vec);

                        Ok(())
                    };
                    Ok(Some(function.into()))
                }
                "PRIMARY" => {
                    unsafe {
                        (&mut *variable_map)
                            .insert("SCREENSHOTS", Variable::SerdeJsonVector(Box::new(vec![])));
                    }
                    let function = move || {
                        let map = unsafe { &mut *variable_map };

                        let image = capturer.capture_primary()?;

                        let mut file_vec = send_screenshots(&[image])?;

                        let vec = match map
                            .get_mut("SCREENSHOTS")
                            .ok_or_else(|| anyhow!("Couldn't get SCREENSHOTS from VariableMap"))?
                        {
                            Variable::SerdeJsonVector(vec) => vec,
                            _ => anyhow::bail!("Variable is not a SerdeJsonVector"),
                        };

                        vec.append(&mut file_vec);

                        Ok(())
                    };

                    Ok(Some(function.into()))
                }
                _ => Ok(None),
            };
        }
        _ => Ok(None),
    }
}

fn parse_iterator(
    lines: &Vec<Vec<&str>>,
    line_pos: &mut usize,
    variable_map: *mut VariableMapType,
) -> anyhow::Result<Iterative> {
    let mut iterative = Iterative::new("".to_owned(), variable_map);

    let mut first_iterative_passed = false;

    while *line_pos < lines.len() {
        let line = &lines[*line_pos];

        let mut i = 0;

        while i < line.len() {
            let word = line[i];
            match word {
                "ITERATE" => {
                    match first_iterative_passed {
                        false => {
                            first_iterative_passed = true;
                            let key = match line.get(i + 1) {
                                Some(key) => key,
                                None => anyhow::bail!(
                            "You haven't provided a Variable to ITERATE\nExample: ITERATE WINDOWS"
                        ),
                            };

                            iterative.change_key(key.to_string());
                        }
                        true => iterative.push(parse_iterator(lines, line_pos, variable_map)?),
                    };
                }
                "IF" => iterative.push(parse_conditional(lines, line_pos, variable_map)?),
                "END" => return Ok(iterative),
                _ => {
                    if let Some(instruction) = parse_instruction(line, variable_map)? {
                        iterative.push(instruction);
                        break;
                    }
                }
            };

            i += 1;
        }

        *line_pos += 1;
    }

    Ok(iterative)
}

fn parse_conditional(
    lines: &Vec<Vec<&str>>,
    line_pos: &mut usize,
    variable_map: *mut VariableMapType,
) -> anyhow::Result<Conditional> {
    #[derive(Debug, Eq, PartialEq)]
    enum Condition {
        If,
        ElseIf,
        Else,
    }
    use Condition::*;

    let mut conditional = Conditional::default();

    let mut first_if_passed = false;

    let mut condition = If;

    let mut else_if_conditional_pos = 0;

    while *line_pos < lines.len() {
        let line = &lines[*line_pos];

        let mut conditions_pos = 0;

        let mut else_if_conditions_pos = 0;

        let mut not_statement = false;

        let mut i = 0;

        while i < line.len() {
            let word = line[i];

            // Returns Option<_>, because Else statements do not have their own conditions.
            let conditions = match condition {
                If => match conditional.conditions.get_mut(conditions_pos) {
                    Some(conditions) => Some(conditions),
                    None => {
                        conditional.conditions.push(Default::default());
                        Some(&mut conditional.conditions[conditions_pos])
                    }
                },
                ElseIf => {
                    let else_if_conditionals = match conditional.else_if_conditionals.as_mut() {
                        Some(else_if_conditionals) => else_if_conditionals,
                        None => {
                            conditional.else_if_conditionals = Some(Default::default());
                            conditional.else_if_conditionals.as_mut().unwrap()
                        }
                    };
                    match else_if_conditionals.get_mut(else_if_conditional_pos) {
                        Some(conditional) => {
                            conditional.conditions.push(Default::default());
                            Some(&mut conditional.conditions[else_if_conditions_pos])
                        }
                        None => {
                            else_if_conditionals.push(Default::default());

                            let else_if_conditionals =
                                conditional.else_if_conditionals.as_mut().unwrap();

                            match else_if_conditionals[else_if_conditional_pos]
                                .conditions
                                .get_mut(else_if_conditions_pos)
                            {
                                Some(conditions) => Some(conditions),
                                None => {
                                    else_if_conditionals[else_if_conditional_pos]
                                        .conditions
                                        .push(Default::default());
                                    Some(
                                        &mut else_if_conditionals[else_if_conditional_pos]
                                            .conditions[else_if_conditions_pos],
                                    )
                                }
                            }
                        }
                    }
                }
                Else => None,
            };

            match word {
                "IF" => match first_if_passed {
                    false => first_if_passed = true,
                    true => {
                        let executable: Box<dyn Executable> =
                            parse_conditional(lines, line_pos, variable_map)?.into();
                        match condition {
                            If => conditional.executables.push(executable),
                            ElseIf => conditional.else_if_conditionals.as_mut().unwrap()
                                [else_if_conditional_pos]
                                .executables
                                .push(executable),
                            Else => conditional
                                .else_executables
                                .as_mut()
                                .unwrap()
                                .push(executable),
                        };
                    }
                },
                "END" => {
                    if condition == If {
                        return Ok(conditional);
                    }
                    condition = If;
                }
                "ELSEIF" => {
                    else_if_conditional_pos = conditional
                        .else_if_conditionals
                        .as_ref()
                        .map(|c| c.len())
                        .unwrap_or(0);
                    condition = ElseIf;
                }
                "ELSE" => {
                    condition = Else;
                    conditional.else_executables = Some(Default::default());
                }
                "NOT" => not_statement = true,
                "OR" => {
                    match condition {
                        If => conditions_pos += 1,
                        ElseIf => else_if_conditions_pos += 1,
                        _ => (),
                    };
                }
                "MATCH" => {
                    let first = line[i - 1];

                    let second = line[i + 1];

                    let (is_string_first, first) = transform_str(first);

                    let conditional_fn: ConditionalFn = match second {
                        "IN" => {
                            let regex_vec = parse_regex_array(line[i + 2])?;

                            i += 1;

                            Box::new(move || {
                                if is_string_first {
                                    for regex in &regex_vec {
                                        debug!("{} {:?} {}", first, regex, regex.is_match(&first));
                                        if regex.is_match(&first) {
                                            return true;
                                        }
                                    }
                                    return false;
                                }

                                let map = unsafe { &*variable_map };

                                let execute = |string: &str| -> bool {
                                    for regex in &regex_vec {
                                        if regex.is_match(string) {
                                            return true;
                                        }
                                    }
                                    false
                                };

                                match map.get(first.as_str()) {
                                    Some(Variable::RcStr(string)) => execute(string),
                                    Some(Variable::ArcStr(string)) => execute(string),
                                    _ => false,
                                }
                            })
                        }
                        _ => {
                            let regex = Regex::new(&second[1..second.len() - 1])?;

                            Box::new(move || {
                                if is_string_first {
                                    return regex.is_match(&first);
                                }

                                let map = unsafe { &*variable_map };

                                if let Some(Variable::RcStr(string)) = map.get(first.as_str()) {
                                    return regex.is_match(string);
                                }

                                false
                            })
                        }
                    };

                    if let Some(conditions) = conditions {
                        conditions.push(conditional_fn)
                    }
                }
                "BIGGER" => {
                    let (is_string_first, first) = transform_str(line[i - 1]);

                    let (is_string_second, second) = transform_str(line[i + 1]);

                    // -- ToDo
                    // Instead of parsing the string as a number
                    // everytime maybe parse it beforehand and just pass the number
                    let conditional_fn = Box::new(move || {
                        use Variable::*;

                        let map = unsafe { &*variable_map };

                        if is_string_first && is_string_second {
                            let first_variable: usize = match first.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            let second_variable: usize = match second.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            return first_variable > second_variable;
                        } else if !is_string_first && is_string_second {
                            let first_variable = match map.get(first.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            let second_variable: usize = match second.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            return match *first_variable {
                                Int(i) => i > second_variable,
                                U64(i) => i > second_variable as u64,
                                Float(i) => i > second_variable as f32,
                                _ => false,
                            };
                        } else if is_string_first && !is_string_second {
                            let first_variable: usize = match first.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            let second_variable = match map.get(second.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            return match *second_variable {
                                Int(i) => i > first_variable,
                                U64(i) => i > first_variable as u64,
                                Float(i) => i > first_variable as f32,
                                _ => false,
                            };
                        } else {
                            let first_variable = match map.get(first.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            let second_variable = match map.get(second.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            return first_variable > second_variable;
                        }
                    });

                    if let Some(conditions) = conditions {
                        conditions.push(conditional_fn);
                    }
                }
                "LESSER" => {
                    let (is_string_first, first) = transform_str(line[i - 1]);

                    let (is_string_second, second) = transform_str(line[i + 1]);

                    // -- ToDo
                    // Instead of parsing the string as a number
                    // everytime maybe parse it beforehand and just pass the number
                    let conditional_fn = Box::new(move || {
                        use Variable::*;

                        let map = unsafe { &*variable_map };

                        if is_string_first && is_string_second {
                            let first_variable: usize = match first.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            let second_variable: usize = match second.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };
                            return first_variable < second_variable;
                        } else if !is_string_first && is_string_second {
                            let first_variable = match map.get(first.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            let second_variable: usize = match second.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            return match *first_variable {
                                Int(i) => i < second_variable,
                                U64(i) => i < second_variable as u64,
                                Float(i) => i < second_variable as f32,
                                _ => false,
                            };
                        } else if is_string_first && !is_string_second {
                            let first_variable: usize = match first.as_str().parse() {
                                Ok(variable) => variable,
                                Err(_) => return false,
                            };

                            let second_variable = match map.get(second.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            return match *second_variable {
                                Int(i) => i < first_variable,
                                U64(i) => i < first_variable as u64,
                                Float(i) => i < first_variable as f32,
                                _ => false,
                            };
                        } else {
                            let first_variable = match map.get(first.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            let second_variable = match map.get(second.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            return first_variable < second_variable;
                        }
                    });

                    if let Some(conditions) = conditions {
                        conditions.push(conditional_fn);
                    }
                }
                "EQ" => {
                    let space = if not_statement { 2 } else { 1 };

                    let first = line[i - space];

                    let is_string_first = is_string(first);

                    let first = if is_string_first {
                        first[1..first.len() - 1].to_owned()
                    } else {
                        first.to_owned()
                    };

                    let second = line[i + space];

                    let is_string_second = is_string(second);

                    let second = if is_string_second {
                        second[1..second.len() - 1].to_owned()
                    } else {
                        second.to_owned()
                    };

                    let conditional_fn: ConditionalFn = Box::new(move || {
                        let map = unsafe { &*variable_map };

                        if is_string_first && !is_string_second {
                            if let Some(variable) = map.get(second.as_str()) {
                                return *variable == first;
                            }
                            false
                        } else if !is_string_first && is_string_second {
                            if let Some(variable) = map.get(first.as_str()) {
                                return *variable == second;
                            }
                            false
                        } else if is_string_first && is_string_second {
                            first == second
                        } else {
                            let first = match map.get(first.as_str()) {
                                Some(first) => first,
                                None => return false,
                            };

                            let second = match map.get(second.as_str()) {
                                Some(second) => second,
                                None => return false,
                            };

                            *first == *second
                        }
                    });

                    if let Some(conditions) = conditions {
                        conditions.push(conditional_fn);
                    }
                }
                "IN" => {
                    let space = if not_statement { 2 } else { 1 };

                    i -= space;

                    let (is_string_first, first) = transform_str(line[i]);

                    i += space + 1;

                    let second = line[i];

                    let conditional_fn: ConditionalFn = if second.starts_with('[') {
                        let vec = parse_array(second)?;
                        Box::new(move || {
                            let mut boolean = false;

                            for string in &vec {
                                if is_string(string) && is_string_first {
                                    if *string == first {
                                        boolean = true;
                                        break;
                                    }
                                    continue;
                                }

                                let map = unsafe { &*variable_map };

                                if let Some(variable) = map.get(string.as_str()) {
                                    if is_string_first {
                                        if *variable == first {
                                            boolean = true;
                                            break;
                                        }
                                        continue;
                                    }

                                    if let Some(second_variable) = map.get(first.as_str()) {
                                        if *variable == *second_variable {
                                            boolean = true;
                                            break;
                                        }
                                    }
                                }
                            }

                            if not_statement {
                                return !boolean;
                            }

                            boolean
                        })
                    } else {
                        let second = second.to_owned();
                        Box::new(move || {
                            let map = unsafe { &*variable_map };

                            let variable = match map.get(second.as_str()) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            let vec = match variable {
                                Variable::Vector(vec) => vec,
                                _ => return false,
                            };

                            let mut boolean = false;

                            for variable in vec.iter() {
                                if is_string_first {
                                    if *variable == first {
                                        boolean = true;
                                        break;
                                    }
                                    continue;
                                }

                                if let Some(second_variable) = map.get(first.as_str()) {
                                    if *variable == *second_variable {
                                        boolean = true;
                                        break;
                                    }
                                }
                            }

                            if not_statement {
                                return !boolean;
                            }

                            boolean
                        })
                    };

                    if let Some(conditions) = conditions {
                        conditions.push(conditional_fn);
                    }
                }
                "ITERATE" => {
                    let iterator = parse_iterator(&lines, line_pos, variable_map)?.into();
                    match condition {
                        If => conditional.executables.push(iterator),
                        ElseIf => conditional.else_if_conditionals.as_mut().unwrap()
                            [else_if_conditional_pos]
                            .executables
                            .push(iterator),
                        Else => conditional
                            .else_executables
                            .as_mut()
                            .unwrap()
                            .push(iterator),
                    };
                }
                _ => {
                    if let Some(instruction) = parse_instruction(&line, variable_map)? {
                        match condition {
                            If => conditional.executables.push(instruction.into()),
                            ElseIf => conditional.else_if_conditionals.as_mut().unwrap()
                                [else_if_conditional_pos]
                                .executables
                                .push(instruction.into()),
                            Else => conditional
                                .else_executables
                                .as_mut()
                                .unwrap()
                                .push(instruction.into()),
                        };
                        break;
                    }
                }
            };

            i += 1;
        }

        *line_pos += 1;
    }

    Ok(conditional)
}

/// Used to determine if passed value is a string and not a variable.
fn is_string(string: impl AsRef<str>) -> bool {
    let string = string.as_ref();
    string.starts_with('\"') || string.starts_with('\'')
}

/// Used to transform a &str into an owned string and remove **"** or **'** signs from the string
/// if needed and it returns if the value was a string or a reference to the map.
fn transform_str(string: impl AsRef<str>) -> (bool, String) {
    let string = string.as_ref();
    let is_string = is_string(string);
    let string = if is_string {
        string[1..string.len() - 1].to_owned()
    } else {
        string.to_owned()
    };
    (is_string, string)
}

fn is_array(string: impl AsRef<str>) -> bool {
    let string = string.as_ref();
    string.starts_with("[") && string.ends_with("]")
}

fn parse_array(string: impl AsRef<str>) -> anyhow::Result<Vec<String>> {
    let string = string.as_ref();
    if !is_array(string) {
        anyhow::bail!("You have an error in your array syntax: {}", string);
    }
    Ok(string[1..string.len() - 1]
        .replace(' ', "")
        .split(',')
        .map(|s| s[1..s.len() - 1].to_owned())
        .collect())
}

fn parse_regex_array(string: impl AsRef<str>) -> Result<Vec<Regex>, regex::Error> {
    let string = string.as_ref();
    let mut vec = vec![];
    for string in string[1..string.len() - 1].replace(' ', "").split(',') {
        vec.push(Regex::new(&string[1..string.len() - 1])?);
    }

    Ok(vec)
}

fn get_word_positions(line: &str) -> Vec<(usize, usize)> {
    let mut word_positions = vec![];
    let mut string_literal: Option<char> = None;
    let mut previous_position = 0;
    for (i, c) in line.chars().enumerate() {
        match c {
            ' ' if string_literal.is_none() => {
                if !&line[previous_position..i].is_empty() {
                    word_positions.push((previous_position, i));
                }
                previous_position = i + 1;
            }
            '[' => {
                string_literal = Some(c);
                previous_position = i;
            }
            ']' => {
                if let Some('[') = string_literal {
                    word_positions.push((previous_position, i + 1));
                    string_literal = None;
                }
                previous_position = i;
            }
            '\"' | '\'' | '`' => {
                match string_literal {
                    Some(a) if a == c => {
                        word_positions.push((previous_position, i + 1));
                        previous_position = i + 1;
                        string_literal = None;
                    }
                    None => string_literal = Some(c),
                    _ => (),
                };
            }
            _ => (),
        };
    }
    if previous_position < line.len() - 1 {
        word_positions.push((previous_position, line.len()));
    }
    word_positions
}
