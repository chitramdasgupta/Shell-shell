use crate::utils::{ensure_file_exists_for_redirection, write_or_append_to_file};
use std::io::Write;
use std::process::exit;
use std::{env, fs};

#[derive(Debug, PartialEq, Eq)]
pub struct Redirection {
    pub kind: RedirectionKind,
    pub channel: OutputChannel,
    pub file: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OutputChannel {
    Stdout,
    Stderr,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectionKind {
    Redirect,
    Append,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Echo {
        args: Vec<String>,
        redirection: Option<Redirection>,
    },
    Exit {
        _arg: i32,
    },
    Type {
        arg: String,
        redirection: Option<Redirection>,
    },
    External {
        name: String,
        args: Vec<String>,
        redirection: Option<Redirection>,
    },
    Pwd {
        redirection: Option<Redirection>,
    },
    Cd {
        arg: String,
    },
    Cat {
        args: Vec<String>,
        redirection: Option<Redirection>,
    },
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

pub struct CommandOutput {
    pub message: String,
    pub channel: OutputChannel,
}

impl CommandOutput {
    pub fn write(&self, redirection: &Option<Redirection>) {
        if redirection.is_none() {
            print!("{}", self.message);
            return;
        }

        let redirection = redirection.as_ref().unwrap();
        if self.channel == redirection.channel && !self.message.is_empty() {
            write_or_append_to_file(&self.message, redirection);
            return;
        }

        ensure_file_exists_for_redirection(redirection);
        print!("{}", self.message);
    }
}

pub trait Executable {
    fn execute(&self);
}

impl Executable for Command {
    fn execute(&self) {
        match self {
            Command::Echo { args, redirection } => {
                CommandOutput {
                    message: format!("{}\n", args.join(" ")),
                    channel: OutputChannel::Stdout,
                }
                .write(redirection);
            }
            Command::Exit { _arg: _ } => {
                exit(0);
            }
            Command::Type { arg, redirection } => {
                let output = if Command::is_builtin(arg) {
                    format!("{arg} is a shell builtin\n")
                } else if let Ok(path) = Command::arg_check_in_path(arg) {
                    format!("{arg} is {path}\n")
                } else {
                    format!("{arg}: not found\n")
                };

                CommandOutput {
                    message: output,
                    channel: OutputChannel::Stdout,
                }
                .write(redirection);
            }
            Command::External {
                name,
                args,
                redirection,
            } => {
                Command::arg_check_in_path(name)
                    .map(|_path| {
                        let output = std::process::Command::new(name)
                            .args(args.clone())
                            .output()
                            .unwrap();

                        if output.status.success() {
                            CommandOutput {
                                message: format!("{}", String::from_utf8_lossy(&output.stdout)),
                                channel: OutputChannel::Stdout,
                            }
                            .write(redirection);
                        } else {
                            CommandOutput {
                                message: format!("{}", String::from_utf8_lossy(&output.stderr)),
                                channel: OutputChannel::Stderr,
                            }
                            .write(redirection);
                        }
                    })
                    .unwrap_or_else(|_| {
                        CommandOutput {
                            message: format!("{}: command not found\n", name),
                            channel: OutputChannel::Stderr,
                        }
                        .write(redirection);
                    });
            }
            Command::Pwd { redirection } => {
                CommandOutput {
                    message: format!("{}\n", env::current_dir().unwrap().display()),
                    channel: OutputChannel::Stdout,
                }
                .write(redirection);
            }
            Command::Cd { arg } => {
                let result = env::set_current_dir(&arg);
                if let Err(_e) = result {
                    println!("cd: {arg}: No such file or directory");
                }
            }
            Command::Cat { args, redirection } => {
                let mut output = String::new();
                let mut error = String::new();
                for file in args.iter() {
                    if fs::metadata(file).is_err() {
                        error = format!("cat: {}: No such file or directory\n", file);
                    } else {
                        output.push_str(&fs::read_to_string(file).unwrap());
                    }
                }

                if redirection.is_none() {
                    if !error.is_empty() {
                        print!("{}", error);
                    } else {
                        print!("{}", output);
                    }
                    return;
                }

                let redirection = redirection.as_ref().unwrap();

                ensure_file_exists_for_redirection(redirection);
                if !error.is_empty() && redirection.channel == OutputChannel::Stderr {
                    write_or_append_to_file(&error, &redirection);
                    print!("{}", output);
                    return;
                }
                if !output.is_empty() && redirection.channel == OutputChannel::Stdout {
                    write_or_append_to_file(&output, &redirection);
                    print!("{}", error);
                }
            }
        }
    }
}
