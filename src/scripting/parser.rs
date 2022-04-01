use crate::scripting::ConditionalFn;

use super::{Conditional, Executable, Instruction};

pub fn parse<'a>(string: impl Into<String>) -> Vec<Box<dyn Executable>> {
    println!("Parsing");

    let mut executables: Vec<Box<dyn Executable>> = vec![];

    let mut string = string.into().replace("\t", " ");

    let newlines: Vec<&str> = string.split("\n").collect();

    let mut line_pos = 0;

    while line_pos < newlines.len() {
        let line = &newlines[line_pos];
        
        line_pos += 1;
        
        if line.is_empty() {
            continue;
        }

        let word = &line[0..2];

        match word {
            "IF" => {
                println!("Parsing Conditional");
                executables.push(Box::new(parse_conditional(&newlines, &mut line_pos)));
            }
            _ => {
                println!("Parsing Instruction");
                let word_positions = get_word_positions(line);
                if let Some(instruction) = parse_instruction(line, &word_positions) {
                    executables.push(Box::new(instruction));
                }
            }
        };
    }

    executables
}

fn parse_instruction(line: &str, word_positions: &Vec<(usize, usize)>) -> Option<Instruction> {
    let (start, end) = *word_positions.get(0)?;
    let word = &line[start..end];
    
    match word {
        "PRINT" => {
            let (start, end) = word_positions[1];
            let word = line[start..end].to_owned();
            let function = move || -> anyhow::Result<()> {
                println!("{}", word);
                Ok(())
            };
            Some(function.into())
        }
        _ => None,
    }
}

fn parse_conditional(lines: &[&str], line_pos: &mut usize) -> Conditional {
    #[derive(Debug)]
    enum Condition {
        If,
        ElseIf,
        Else,
    };
    use Condition::*;

    let mut conditional = Conditional::default();

    let mut condition = If;

    while *line_pos < lines.len() {
        let line = lines[*line_pos];

        if line.len() < 2 {
            break;
        }

        let word_positions = get_word_positions(line);

        let mut conditions_pos = 0;

        let mut else_if_conditional_pos = 0;

        let mut else_if_conditions_pos = 0;

        let mut not_statement = false;

        let mut or_statement = false;

        let mut i = 0;

        while i < word_positions.len() {
            let (start, end) = word_positions[i];
            let word = &line[start..end];
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

                    let (start, end) = word_positions[i];
                    let first = line[start..end].to_owned();

                    i += space + 1;

                    let (start, end) = word_positions[i];
                    let second = line[start..end].to_owned();

                    let conditional_fn: ConditionalFn = if not_statement {
                        Box::new(move || first != second)
                    } else {
                        Box::new(move || first == second )
                    };

                    let conditions: &mut Vec<ConditionalFn> = match condition {
                        If => match conditional.conditions.get_mut(conditions_pos) {
                            Some(conditions) => conditions,
                            None => {
                                conditional.conditions.push(Default::default());
                                &mut conditional.conditions[conditions_pos]
                            }
                        },
                        ElseIf => {
                            let else_if_conditionals =
                                match conditional.else_if_conditionals.as_mut() {
                                    Some(else_if_conditionals) => else_if_conditionals,
                                    None => {
                                        conditional.else_if_conditionals = Some(Default::default());
                                        conditional.else_if_conditionals.as_mut().unwrap()
                                    }
                                };
                            match else_if_conditionals.get_mut(else_if_conditional_pos) {
                                Some(conditional) => {
                                    &mut conditional.conditions[else_if_conditions_pos]
                                }
                                None => {
                                    else_if_conditionals.push(Conditional::default());

                                    let else_if_conditionals =
                                        conditional.else_if_conditionals.as_mut().unwrap();

                                    match else_if_conditionals[else_if_conditional_pos]
                                        .conditions
                                        .get_mut(else_if_conditions_pos)
                                    {
                                        Some(conditions) => conditions,
                                        None => {
                                            else_if_conditionals[else_if_conditional_pos]
                                                .conditions
                                                .push(vec![]);

                                            &mut else_if_conditionals[else_if_conditional_pos]
                                                .conditions[else_if_conditions_pos]
                                        }
                                    }
                                }
                            }
                        }
                        _ => continue,
                    };

                    conditions.push(conditional_fn);
                }
                _ => {
                    if let Some(instruction) = parse_instruction(line, &word_positions) {
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
            '\"' | '\'' => {
                match string_literal {
                    Some(a) if a == c => {
                        word_positions.push((previous_position + 1, i));
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
    if word_positions.is_empty() {
        word_positions.push((0, line.len()));
    }
    word_positions
}
