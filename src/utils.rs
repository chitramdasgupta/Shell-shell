use std::env;

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
