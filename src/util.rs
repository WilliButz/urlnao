use crate::config::Config;

use uuid::Uuid;
use std::fs;

pub fn prepend_tmp_dir(s: &str) -> String {
    format!("tmp/{}", s)
}

pub fn prepend_upload_dir(s: &str) -> String {
    format!("uploads/{}", s)
}

pub fn new_random_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn cleanup(config: &Config) {
    if let Err(_) = fs::remove_file(config.socket_path.to_string()) {
        eprintln!("failed to cleanup socket {}", config.socket_path);
    }
}
