mod command;
mod parser;
mod utils;

use crate::command::Executable;
use std::io;
use std::io::{BufRead, Write};

fn main() {
    loop {
        display_command_prompt();

        let stdin = io::stdin().lock();
        let line = stdin.lines().next().unwrap().unwrap();
        let command = parser::parse_command(&line);

        command.execute();
    }
}

fn display_command_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}
