use std::env;

pub fn get(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!("Environment variable {} not set", key)
    })
}