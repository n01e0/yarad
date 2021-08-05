use serde::Deserialize;
use std::convert::{From, TryFrom};
use std::fs::{read_to_string, File, OpenOptions};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    log_file: String,
    log_level: String,
    local_socket: String,
    local_socket_group: String,
    local_socket_mode: String,
    rules_dir: String,
    user: String,
}

#[derive(Debug)]
struct Config {
    local_socket: String,
    rules_dir: String,
    user: String,
    log_path: String,
}
