#![allow(dead_code)]

use sha2::{
    Digest,
    Sha256,
};
use std::{
    io,
    fs::{OpenOptions, create_dir_all},
    process::Command,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    thread,
    time,
};
use zip::ZipArchive;
use crate::constants::NO_WINDOW;

pub fn verify_file<P: AsRef<Path>>(filepath: P, hash: &[u8]) -> io::Result<bool> {
    match hash_file(filepath) {
        Ok(file_hash) => return Ok(&file_hash == hash),
        Err(error) => return Err(error),
    }
}

pub fn hash_file<P: AsRef<Path>>(filepath: P) -> io::Result<Vec<u8>> {
    let mut file = OpenOptions::new().read(true).open(filepath)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let _hash = hasher.finalize();
    let _hash_array: &[u8] = _hash.as_ref();
    return Ok(_hash_array.to_owned());
}

pub fn download_file(path: Option<&Path>, link: &str, filename: &Path) -> io::Result<()> {
    let command: String = match path {
        Some(path) => {
            if !path.is_dir() {
                create_dir_all(path)?
            }
            format!("cd /D {} && curl --retry 3 --location {} --output {}", path.display(), link, filename.display())    
        },
        None => {
            format!("curl -L {} --output {}", link, filename.display())
        }
    };
    let mut child =Command::new("cmd")
    .creation_flags(NO_WINDOW)
    .args(["/C", &command])
    .spawn()?;
    let _result = child.wait()?;
    return Ok(());
}

pub fn extract_zip<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> io::Result<()> {
    let file = OpenOptions::new().read(true).open(source)?;
    let mut zip = ZipArchive::new(file)?;
    zip.extract(destination)?;
    return Ok(());
}

pub fn thread_sleep(seconds: f64) {
    thread::sleep(time::Duration::from_secs_f64(seconds));
}

pub fn cmd_with_output(command: &str) -> io::Result<String> {
    let command_output = Command::new("cmd").creation_flags(NO_WINDOW).args(["/C", &command]).output()?;
    return Ok(String::from_utf8_lossy(&command_output.stdout).trim().to_owned());
}

pub fn cmd_without_output(command: &str) -> io::Result<()> {
    Command::new("CMD").creation_flags(NO_WINDOW).args(["/C", command]).spawn()?;
    return Ok(());
}

pub fn replace_homedrive(path : &mut PathBuf, HOMEDRIVE : &str) {
    let path_str = path.to_str().unwrap_or("");
    if path_str.contains("$HOMEDRIVE") {*path = PathBuf::from(path_str.replace("$HOMEDRIVE", HOMEDRIVE));}
}

pub fn replace_username(path : &mut PathBuf, USERNAME : &str) {
    let path_str = path.to_str().unwrap_or("");
    if path_str.contains("$USERNAME") {*path = PathBuf::from(path_str.replace("$USERNAME", USERNAME));}
}