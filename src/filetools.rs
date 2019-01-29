use std::fs;
use std::path::{Path};
use std::str::FromStr;

pub fn read_value_from_file<P: AsRef<Path>, V: FromStr>(path: P, index: usize) -> Option<V> {
    let contents = fs::read_to_string(path)
        .unwrap_or(String::new());
    let values: Vec<&str> = contents.split(" ").collect();

    let value_str = match values.get(index) {
        Some(v) => v,
        None => "0",
    };
    
    match value_str.trim().parse() {
        Ok(v) => Some(v),
        Err(_) => None
    }
}
