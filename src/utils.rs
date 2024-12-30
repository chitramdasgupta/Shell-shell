use crate::command::{Redirection, RedirectionChannel, RedirectionKind};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::{env, fs};

pub fn expand_home_path(path: &str) -> String {
    if path.as_bytes().get(0) == Some(&b'~') {
        let home_dir = env::var("HOME").unwrap();

        let mut expanded_path = path.to_string();
        expanded_path.remove(0);
        expanded_path.insert_str(0, home_dir.as_str());

        expanded_path
    } else {
        path.to_string()
    }
}

/// This takes the output of a command, and an optional redirection Command, and a flag to indicate
/// whether the main command was successful or not
/// If there is a redirection operator and the success status of the command and the redirection operator match
/// then the output message is sent to the file, else printed out
/// If there is no redirection it simply prints to stdout
pub fn write_output(output: &str, redirection: &Option<Redirection>, success: bool) {
    if let Some(redirection) = redirection {
        if redirection.kind == RedirectionKind::Redirect
            || (redirection.kind == RedirectionKind::Append
                && fs::metadata(&redirection.file).is_err())
        {
            fs::write(&redirection.file, String::new()).unwrap();
        }

        if (success && redirection.channel == RedirectionChannel::Stdout)
            || (!success && redirection.channel == RedirectionChannel::Stderr)
        {
            if redirection.kind == RedirectionKind::Redirect {
                fs::write(&redirection.file, output).unwrap();
            } else {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&redirection.file)
                    .unwrap();

                file.seek(SeekFrom::End(0)).unwrap();
                file.write(output.as_bytes()).unwrap();
            }
            return;
        } else {
            print!("{}", output);
        }
    } else {
        print!("{}", output);
    }
}
