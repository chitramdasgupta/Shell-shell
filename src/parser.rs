use crate::command::Command;
use crate::utils::expand_home_path;

pub fn parse_command(line: &str) -> Command {
    let tokens = tokenize(line);
    parse(&tokens)
}

fn tokenize(input: &str) -> Vec<String> {
    let input = input.trim();

    let mut tokens: Vec<String> = Vec::new();
    let mut curr = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut to_escape = false;
    for c in input.chars() {
        if to_escape {
            if in_double_quote && (c == '"' || c == '\\' || c == '`' || c == '$') {
                curr.push(c);
                to_escape = false;
            } else if in_double_quote {
                curr.push('\\');
                curr.push(c);
                to_escape = false;
            } else if !in_double_quote && !in_double_quote {
                curr.push(c);
                to_escape = false;
            }

            continue;
        }

        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        }

        if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        }

        if c == '\\' && ((!in_single_quote && !in_double_quote) || in_double_quote) {
            to_escape = true;
            continue;
        }

        if c.is_whitespace() && !in_single_quote && !in_double_quote {
            if !curr.is_empty() {
                tokens.push(curr.clone());
                curr.clear();
            }
        } else {
            curr.push(c);
        }
    }

    if !curr.is_empty() {
        tokens.push(curr.clone());
    }

    tokens
}

fn parse(tokens: &Vec<String>) -> Command {
    match tokens[0].as_str() {
        "echo" => Command::Echo {
            args: tokens[1..].to_vec(),
        },
        "exit" => Command::Exit {
            _arg: if tokens.len() > 1 {
                tokens[1].parse().unwrap()
            } else {
                0
            },
        },
        "type" => Command::Type {
            arg: tokens[1].parse().unwrap(),
        },
        "pwd" => Command::Pwd {},
        "cd" => Command::Cd {
            arg: expand_home_path(&tokens[1]),
        },
        "cat" => {
            let destinations: Vec<String> = tokens[1..]
                .iter()
                .map(|path| expand_home_path(path))
                .collect();

            Command::Cat { args: destinations }
        }
        _ => Command::External {
            name: tokens[0].to_string(),
            args: tokens[1..].to_vec(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_echo_hello_world_simple() {
        let input = "echo hello world";
        let expected = vec!["echo".to_string(), "hello".to_string(), "world".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_echo_hello_world_with_spaces() {
        let input = "echo hello    world";
        let expected = vec!["echo".to_string(), "hello".to_string(), "world".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_type_echo() {
        let input = "type echo";
        let expected = vec!["type".to_string(), "echo".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_ls() {
        let input = "ls";
        let expected = vec!["ls".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_cd() {
        let input = "cd ~/Documents";
        let expected = vec!["cd".to_string(), "~/Documents".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_command_with_single_quote() {
        let input = "echo 'world     test'";
        let expected = vec!["echo".to_string(), "world     test".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_command_with_double_quote() {
        let input = r#"echo "bar    bar"  "shell's"  "foo""#;
        let expected = vec![
            "echo".to_string(),
            "bar    bar".to_string(),
            "shell's".to_string(),
            "foo".to_string(),
        ];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_non_quoted_backslash() {
        let input = r"echo hello\ \ \ \ \ \ world";
        let expected = vec!["echo".to_string(), "hello      world".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_with_backslash_inside_double_quotes() {
        let input = r#"echo "hello\"insidequotes"script\""#;
        let expected = vec![
            "echo".to_string(),
            "hello\"insidequotesscript\"".to_string(),
        ];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_with_backslash_inside_single_quotes_inside_double_quotes() {
        let input = r#"echo "hello'script'\\n'world""#;
        let expected = vec!["echo".to_string(), r"hello'script'\n'world".to_string()];

        let result = tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_command_echo_hello_world() {
        let input = "echo hello     world";
        let expected = Command::Echo {
            args: vec!["hello".to_string(), "world".to_string()],
        };

        let result = parse_command(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_command_exit() {
        let input = "exit 0";
        let expected = Command::Exit { _arg: 0 };

        let result = parse_command(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_command_type_echo() {
        let input = "type echo";
        let expected = Command::Type {
            arg: "echo".to_string(),
        };

        let result = parse_command(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_command_ls() {
        let input = "ls";
        let expected = Command::External {
            name: "ls".to_string(),
            args: vec![],
        };

        let result = parse_command(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_command_cd() {
        let input = "cd ~/Documents";
        let expected = Command::Cd {
            arg: "/home/cdg/Documents".to_string(),
        };

        let result = parse_command(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_command_cat_with_quoted_file_names() {
        let input = r#"cat "/tmp/bar/f\n41" "/tmp/bar/f\10" "/tmp/bar/f'\'62""#;
        let expected = Command::Cat {
            args: vec![
                r"/tmp/bar/f\n41".to_string(),
                r"/tmp/bar/f\10".to_string(),
                r"/tmp/bar/f'\'62".to_string(),
            ],
        };

        let result = parse_command(input);
        assert_eq!(result, expected);
    }
}
