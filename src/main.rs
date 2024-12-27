use std::env::var;
use std::{env, fs};
use std::io::{self, BufRead, Write};

#[derive(Debug)]
enum Command {
    EXIT(String),
    ECHO(String, String),
    INVALID(String),
    TYPE(String, Box<Command>),
    PWD(String, String),
}

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin().lock();
    for line in stdin.lines() {
        let input = line.unwrap();
        let split = input.trim().split(" ").collect::<Vec<&str>>();

        let command = parse_command(split);
        if let Ok(Command::EXIT(_)) = handle_command(command) {
            return;
        }

        print!("$ ");
        io::stdout().flush().unwrap();
    }
}

fn handle_command(command: Command) -> Result<Command, String> {
    match command {
        Command::INVALID(command) => {
            let split = command.split(" ").collect::<Vec<&str>>();

            let option = is_present_in_path(&*var("PATH").unwrap(), &split[0]);
            if let Ok(path) = option {
                let mut new_command = std::process::Command::new(path);
                if split.len() > 1 {
                    new_command.args(split[1..].to_vec());
                }
                let output = new_command.output().unwrap(); // Assuming that the child process will always be successful

                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    print!("{stdout}");
                }
            } else {
                println!("{command}: command not found");
            }

            Ok(Command::INVALID(command))
        }
        Command::EXIT(_) => Ok(Command::EXIT("".to_string())),
        Command::ECHO(command, text) => {
            println!("{}", text);
            Ok(Command::ECHO(command, text))
        }
        Command::TYPE(name, sub_command) => {
            if is_builtin(&name) {
                println!("{name} is a shell builtin");
                return Ok(Command::TYPE(name, sub_command));
            }

            let option = is_present_in_path(&*var("PATH").unwrap(), &*name);
            if let Ok(path) = option {
                println!("{name} is {path}");
                return Ok(Command::TYPE(name, sub_command));
            }

            println!("{name}: not found");

            Ok(Command::TYPE(name, sub_command))
        }
        Command::PWD(name, cwd) => {
            println!("{cwd}");
            Ok(Command::PWD(name, cwd))
        }
    }
}

fn parse_command(input: Vec<&str>) -> Command {
    if input.len() == 0 {
        return Command::INVALID("".to_string());
    }

    match input[0] {
        "exit" => Command::EXIT(input.get(1).unwrap_or(&"").to_string()),
        "echo" => Command::ECHO(input[0].to_string(), input[1..].join(" ")),
        "type" => {
            if input.len() < 2 {
                Command::INVALID("type: missing argument".to_string())
            } else {
                Command::TYPE(
                    input[1].to_string(),
                    Box::new(Command::INVALID("".to_string())),
                )
            }
        },
        "pwd" => {
            let cwd = env::current_dir().unwrap().display().to_string();
            Command::PWD(input[0].to_string(), cwd)
        }
        _ => Command::INVALID(input.join(" ")),
    }
}

fn is_builtin(command: &str) -> bool {
    matches!(command, "exit" | "echo" | "type" | "pwd")
}

fn is_present_in_path(path: &str, program: &str) -> Result<String, String> {
    let directories = path.split(":").collect::<Vec<_>>();
    for directory in directories {
        match fs::read_dir(directory) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.file_name() == program {
                            return Ok(entry.path().display().to_string());
                        }
                    }
                }
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }

    Err(format!("{} not found", program))
}
