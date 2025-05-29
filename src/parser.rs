#![allow(unused)]
use shellwords;

use crate::ast::{CommandLine, CommandSegment, Pipeline, Redirection};

pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
    shellwords::split(input).map_err(|e| e.to_string())
}

// pub fn parse_command_line(input: &str) -> Result<CommandLine, String> {
//     let tokens = tokenize(input)?;
//     let mut pipelines = Vec::new();
//     let mut current_pipeline = Vec::new();
//
//     for token in tokens {
//         if token == ";" || token == "&&" || token == "||" {
//             if !current_pipeline.is_empty() {
//                 pipelines.push(parse_pipeline(&current_pipeline)?);
//                 current_pipeline.clear();
//             }
//         } else {
//             current_pipeline.push(token);
//         }
//     }
//
//     if !current_pipeline.is_empty() {
//         pipelines.push(parse_pipeline(&current_pipeline)?);
//     }
//
//     Ok(CommandLine { pipelines })
// }

pub fn parse_command_segment(tokens: &[String]) -> Result<CommandSegment, String> {
    if tokens.is_empty() {
        return Err("No tokens provided".to_string());
    }

    let cmd = tokens[0].clone();
    let mut args = Vec::new();
    let mut redirections = Vec::new();

    let mut tokens = tokens[1..].iter().peekable();
    while let Some(token) = tokens.next() {
        match token.as_str() {
            ">" | "1>" => {
                if let Some(next) = tokens.next() {
                    redirections.push(Redirection::Stdout(next.clone()));
                } else {
                    return Err("Expected filename after '>'".to_string());
                }
            }
            ">>" | "1>>" => {
                if let Some(next) = tokens.next() {
                    redirections.push(Redirection::StdoutAppend(next.clone()));
                } else {
                    return Err("Expected filename after '>>'".to_string());
                }
            }
            "2>" => {
                if let Some(next) = tokens.next() {
                    redirections.push(Redirection::Stderr(next.clone()));
                } else {
                    return Err("Expected filename after '2>'".to_string());
                }
            }
            "2>>" => {
                if let Some(next) = tokens.next() {
                    redirections.push(Redirection::StderrAppend(next.clone()));
                } else {
                    return Err("Expected filename after '2>>'".to_string());
                }
            }
            _ => args.push(token.clone()),
        }
    }

    Ok(CommandSegment {
        cmd,
        args,
        redirections,
    })
}

pub fn parse_pipeline(input: &str) -> Result<Pipeline, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("No tokens provided".to_string());
    }

    let mut segments = Vec::new();
    let mut current_segment = Vec::new();

    for token in tokens {
        if token == "|" {
            if !current_segment.is_empty() {
                segments.push(parse_command_segment(&current_segment)?);
                current_segment.clear();
            }
        } else {
            current_segment.push(token.clone());
        }
    }

    if !current_segment.is_empty() {
        segments.push(parse_command_segment(&current_segment)?);
    }

    Ok(Pipeline { segments })
}
