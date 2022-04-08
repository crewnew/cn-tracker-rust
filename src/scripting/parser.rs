use super::{Conditional, Executable, Instruction, Variable, VARIABLES_MAP};
use crate::scripting::ConditionalFn;
use anyhow::Context;
use std::cmp::Ordering;

/// Returns the seconds to wait for execution and a vector of executable pointers.
pub fn parse(string: impl AsRef<str>) -> anyhow::Result<(usize, Vec<Box<dyn Executable>>)> {
    VARIABLES_MAP
        .write()
        .unwrap()
        .insert("a".to_owned(), "a".into());
    let mut executables: Vec<Box<dyn Executable>> = vec![];

    let mut timeout: Option<usize> = None;

    let mut string = string.as_ref().replace("\t", " ");

    let lines_split = string.split("\n");

    let mut lines: Vec<Vec<&str>> = vec![];

    for line in lines_split {
        if line.is_empty() {
            lines.push(vec![]);
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

        if line.is_empty() {
            line_pos += 1;
            continue;
        }

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

                let amount: usize = match line[1].parse() {
                    Ok(amount) => amount,
                    Err(_) => {
                        anyhow::bail!(
                            "You haven't provided a valid number as a second parameter at line {}.",
                            line_pos
                        )
                    }
                };

                let time_variant = line[2];

                match time_variant {
                   "MILLISECONDS" => timeout = Some(amount),
                   "SECONDS" => timeout = Some(amount * 1000),
                   "MINUTES" => timeout = Some(amount * 1000 * 60),
                   "HOURS" => timeout = Some(amount * 1000 * 60 * 60),
                   _ => anyhow::bail!("The time variant {} you provided at line {} doesn't exist.\nYour options are: MILLISECONDS, SECONDS, MINUTES and HOURS.", time_variant, line_pos)
                };
            }
            "IF" => {
                executables.push(Box::new(parse_conditional(&lines, &mut line_pos)));
            }
            _ => {
                if let Some(instruction) = parse_instruction(&line) {
                    executables.push(Box::new(instruction));
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

fn parse_instruction(line: &[&str]) -> Option<Instruction> {
    let word = line[0];
    match word {
        "PRINT" => {
            let word = line[1][1..line[1].len() - 1].to_owned();
            let function = move || -> anyhow::Result<()> {
                println!("{}", word);
                Ok(())
            };
            Some(function.into())
        }
        _ => None,
    }
}

fn parse_conditional(lines: &Vec<Vec<&str>>, line_pos: &mut usize) -> Conditional {
    #[derive(Debug)]
    enum Condition {
        If,
        ElseIf,
        Else,
    };
    use Condition::*;

    let mut conditional = Conditional::default();

    let mut condition = If;

    let mut else_if_conditional_pos = 0;

    while *line_pos < lines.len() {
        let line = &lines[*line_pos];

        if line.is_empty() {
            break;
        }

        let mut conditions_pos = 0;

        let mut else_if_conditions_pos = 0;

        let mut not_statement = false;

        let mut i = 0;

        while i < line.len() {
            let word = line[i];

            /// Returns Option<_>, because Else statements do not have their own conditions.
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
                    conditional.else_instructions = Some(Default::default());
                }
                "NOT" => not_statement = true,
                "OR" => {
                    match condition {
                        If => conditions_pos += 1,
                        ElseIf => else_if_conditions_pos += 1,
                        _ => (),
                    };
                }
                "EQ" => {
                    let space = if not_statement { 2 } else { 1 };

                    i -= space;

                    let first = line[i];

                    let is_string_first = is_string(first);

                    let first = if is_string_first {
                        first[1..first.len() - 1].to_owned()
                    } else {
                        first.to_owned()
                    };

                    i += space + 1;

                    let second = line[i];

                    let is_string_second = is_string(second);

                    let second = if is_string_second {
                        second[1..second.len() - 1].to_owned()
                    } else {
                        second.to_owned()
                    };

                    let conditional_fn: ConditionalFn = Box::new(move || {
                        let map = VARIABLES_MAP.read().unwrap();

                        if is_string_first && !is_string_second {
                            if let Some(variable) = map.get(&second) {
                                return *variable == first;
                            }
                            false
                        } else if !is_string_first && is_string_second {
                            if let Some(variable) = map.get(&first) {
                                return *variable == second;
                            }
                            false
                        } else {
                            let first = match map.get(&first) {
                                Some(first) => first,
                                None => return false,
                            };

                            let second = match map.get(&second) {
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

                    let first = line[i].to_owned();

                    i += space + 1;

                    let second = line[i].to_owned();

                    let is_string_first = is_string(&first);

                    let conditional_fn: ConditionalFn = if second.starts_with('[') {
                        let vec = parse_array(second);
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

                                let map = VARIABLES_MAP.read().unwrap();

                                if let Some(variable) = map.get(string) {
                                    if is_string_first {
                                        if *variable == first {
                                            boolean = true;
                                            break;
                                        }
                                        continue;
                                    }

                                    if let Some(second_variable) = map.get(&first) {
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
                        Box::new(move || {
                            let map = VARIABLES_MAP.read().unwrap();

                            let variable = match map.get(&second) {
                                Some(variable) => variable,
                                None => return false,
                            };

                            let vec = match variable {
                                Variable::Vector(vec) => vec,
                                _ => return false,
                            };

                            let mut boolean = false;

                            for variable in vec {
                                if is_string_first {
                                    if *variable == first {
                                        boolean = true;
                                        break;
                                    }
                                    continue;
                                }

                                if let Some(second_variable) = map.get(&first) {
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
                _ => {
                    if let Some(instruction) = parse_instruction(&line) {
                        match condition {
                            If => conditional.instructions.push(instruction),
                            ElseIf => conditional.else_if_conditionals.as_mut().unwrap()
                                [else_if_conditional_pos]
                                .instructions
                                .push(instruction),
                            Else => conditional
                                .else_instructions
                                .as_mut()
                                .unwrap()
                                .push(instruction),
                        };
                        break;
                    }
                }
            };

            i += 1;
        }

        *line_pos += 1;
    }

    conditional
}

/// Used to determine if passed value is a string and not a variable.
fn is_string(string: impl AsRef<str>) -> bool {
    let string = string.as_ref();
    string.starts_with('\"') || string.starts_with('\'')
}

fn parse_array(string: impl AsRef<str>) -> Vec<String> {
    let string = string.as_ref();
    string[1..string.len() - 1]
        .replace(' ', "")
        .split(',')
        .map(|s| s[1..s.len() - 1].to_owned())
        .collect()
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
            '\"' | '\'' => {
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
