use crate::utils::write_output;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use std::{env, fs};

#[derive(Debug, PartialEq, Eq)]
pub struct Redirection {
    pub kind: RedirectionKind,
    pub channel: RedirectionChannel,
    pub file: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectionChannel {
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
                                // return Ok(entry.file_name().to_string_lossy().to_string());
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
            Command::Echo { args, redirection } => {
                write_output(&format!("{}\n", args.join(" ")), redirection, true);
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

                write_output(&output, redirection, true);
            }
            Command::External {
                name,
                args,
                redirection,
            } => {
                Command::arg_check_in_path(name)
                    .map(|path| {
                        // println!("name: {name}");
                        // println!("path: {path}");
                        let output = std::process::Command::new(name)
                            .args(args.clone())
                            .output()
                            .unwrap();

                        if output.status.success() {
                            write_output(
                                &format!("{}", String::from_utf8_lossy(&output.stdout)),
                                redirection,
                                true,
                            )
                        } else {
                            write_output(
                                &format!("{}", String::from_utf8_lossy(&output.stderr)),
                                redirection,
                                false,
                            )
                        }
                    })
                    .unwrap_or_else(|_| {
                        write_output(
                            &format!("{}: command not found\n", name),
                            redirection,
                            false,
                        )
                    });
            }
            Command::Pwd { redirection } => {
                write_output(
                    &format!("{}\n", env::current_dir().unwrap().display()),
                    redirection,
                    true,
                );
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

                if let Some(redirection) = redirection {
                    if redirection.kind == RedirectionKind::Redirect
                        || (redirection.kind == RedirectionKind::Append
                            && fs::exists(&redirection.file).is_err())
                    {
                        fs::write(&redirection.file, String::new()).unwrap();
                    }

                    if !error.is_empty() && redirection.channel == RedirectionChannel::Stderr {
                        if redirection.kind == RedirectionKind::Redirect {
                            fs::write(&redirection.file, &error).unwrap();
                        } else {
                            let mut file = OpenOptions::new()
                                .write(true)
                                .append(true)
                                .open(&redirection.file)
                                .unwrap();

                            file.write_all(error.as_bytes()).unwrap();
                        }

                        print!("{}", output);
                    } else if !error.is_empty() && redirection.channel == RedirectionChannel::Stdout
                    {
                        if redirection.kind == RedirectionKind::Redirect {
                            fs::write(&redirection.file, &output).unwrap();
                        } else {
                            let mut file = OpenOptions::new()
                                .write(true)
                                .append(true)
                                .open(&redirection.file)
                                .unwrap();

                            file.write_all(output.as_bytes()).unwrap();
                        }

                        print!("{}", error);
                    } else if error.is_empty() && redirection.channel == RedirectionChannel::Stdout
                    {
                        if redirection.kind == RedirectionKind::Redirect {
                            fs::write(&redirection.file, &output).unwrap();
                        } else {
                            let mut file = OpenOptions::new()
                                .write(true)
                                .append(true)
                                .open(&redirection.file)
                                .unwrap();

                            file.write_all(output.as_bytes()).unwrap();
                        }
                    } else {
                        print!("{}", output);
                    }
                } else {
                    if !error.is_empty() {
                        print!("{}", error);
                    } else {
                        print!("{}", output);
                    }
                }
            }
        }
    }
}
