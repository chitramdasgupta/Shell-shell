use std::{env, fs};
use std::fs::OpenOptions;
use std::io::Write;
use crate::command::{Redirection, RedirectionKind};

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

pub fn write_or_append_to_file(message: &str, redirection: &Redirection) {
    match redirection.kind {
        RedirectionKind::Redirect => {
            fs::write(&redirection.file, message).unwrap();
        }
        RedirectionKind::Append => {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&redirection.file)
                .unwrap();

            file.write_all(message.as_bytes()).unwrap();
        }
    }
}

pub fn ensure_file_exists_for_redirection(redirection: &Redirection) {
    if redirection.kind == RedirectionKind::Redirect
        || (redirection.kind == RedirectionKind::Append
        && fs::metadata(&redirection.file).is_err())
    {
        fs::write(&redirection.file, String::new()).unwrap();
    }
}
