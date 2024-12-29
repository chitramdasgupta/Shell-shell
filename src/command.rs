use std::process::exit;
use std::{env, fs};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Echo { args: Vec<String> },
    Exit { _arg: i32 },
    Type { arg: String },
    External { name: String, args: Vec<String> },
    Pwd,
    Cd { arg: String },
    Cat { args: Vec<String> },
}

impl Command {
    pub fn is_builtin(arg: &str) -> bool {
        matches!(arg, "echo" | "exit" | "type" | "pwd" | "cd")
    }

    pub fn arg_check_in_path(arg: &str) -> Result<String, String> {
        let path = env::var("PATH").unwrap();

        let directories = path.split(":").collect::<Vec<_>>();
        for directory in directories {
            match fs::read_dir(directory) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if entry.file_name() == arg {
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

        Err(format!("{} not found", arg))
    }
}

pub trait Executable {
    fn execute(&self);
}

impl Executable for Command {
    fn execute(&self) {
        match self {
            Command::Echo { args } => {
                println!("{}", args.join(" "));
            }
            Command::Exit { _arg: _ } => {
                exit(0);
            }
            Command::Type { arg } => {
                if Command::is_builtin(arg) {
                    println!("{arg} is a shell builtin");
                } else if let Ok(path) = Command::arg_check_in_path(arg) {
                    println!("{arg} is {path}");
                } else {
                    println!("{arg}: not found");
                }
            }
            Command::External { name, args } => {
                Command::arg_check_in_path(name)
                    .map(|path| {
                        let output = std::process::Command::new(path)
                            .args(args.clone())
                            .output()
                            .unwrap();

                        if output.status.success() {
                            print!("{}", String::from_utf8_lossy(&output.stdout));
                        }
                    })
                    .unwrap_or_else(|_| println!("{name}: command not found"));
            }
            Command::Pwd => {
                println!("{}", env::current_dir().unwrap().display());
            }
            Command::Cd { arg } => {
                let result = env::set_current_dir(&arg);
                if let Err(_e) = result {
                    println!("cd: {arg}: No such file or directory");
                }
            }
            Command::Cat { args } => {
                let output: String = args
                    .iter()
                    .map(|file| fs::read_to_string(file).unwrap())
                    .collect::<Vec<_>>()
                    .concat();

                print!("{output}");
            }
        }
    }
}
