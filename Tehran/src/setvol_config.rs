use std::{
    io::{self, Read},
    path::PathBuf,
    fs::{OpenOptions, create_dir_all, remove_file, remove_dir}
};
use serde::{
    Deserialize,
    Serialize
};
use crate::utils::{
    extract_zip,
    download_file,
};
use crate::constants::{
    setvol::DEFAULT_DOWNLOAD_LINK,
    TEHRAN_NAME,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SetvolConfig {
    pub filename: PathBuf,
    pub path: PathBuf,
    pub hash: [u8; 32],
    pub archive_path: PathBuf,
    pub archive_filename: PathBuf,
    pub archive_hash: [u8; 32],
    pub archive_link: String,
}

impl SetvolConfig {
    pub fn get_filepath(&self) -> PathBuf {
        return self.path.join(&self.filename);
    }
    pub fn get_archive_filepath(&self) -> PathBuf {
        return self.archive_path.join(&self.archive_filename);
    }
    pub fn install(&self) {
        println!("+-- [INFO] installing setvol executable...");
        match create_dir_all(&self.path) {
            Ok(..) => {
                let archive_filepath: PathBuf = self.get_archive_filepath();
                match extract_zip(&archive_filepath, &self.path) {
                    Ok(..) => {
                        println!("+-- [INFO] done -> {}", self.path.display());
                    },
                    Err(error) => {
                        eprintln!("+-- [WARNING] failed to unzip the setvol backup archive : {} to {}\n+-- ///////// {}", archive_filepath.display(), self.path.display(), error);
                    }
                }
            }
            Err(error) => {
                eprintln!("+-- [WARNING] failed to create the setvol directory : {}\n+-- ///////// {}", self.path.display(), error);
            }
        }
    }
    pub fn uninstall(&self) {
        println!("+-- [INFO] uninstalling setvol executable...");
        let filepath: PathBuf = self.get_filepath();
        match remove_file(&filepath) {
            Ok(..) => println!("+---- [INFO] removed file : {}", self.filename.display()),
            Err(error) => eprintln!("+---- [WARNING] failed to remove file : {}\n+---- ///////// {}", self.filename.display(), error),
        }
        match remove_dir(&self.path) {
            Ok(..) => println!("+---- [INFO] removed directory : {}", self.path.display()),
            Err(error) => eprintln!("+---- [WARNING] failed to remove directory : {}\n+---- ///////// {}", self.path.display(), error),
        }
        println!("+-- [INFO] done");
    }
    pub fn archive_install(&self) {
        println!("+-- [INFO] downloading setvol backup archive...");
        match download_file(Some(&self.archive_path), &self.archive_link, &self.archive_filename) {
            Ok(..) => println!("+-- [INFO] done"),
            Err(error) => eprintln!("+-- [WARNING] failed to download setvol backup archive\n+-- ///////// {}", error)
        }
    }
    pub fn archive_uninstall(&self) {
        println!("+-- [INFO] removing setvol backup archive...");
        let filepath: PathBuf = self.get_archive_filepath();
        match remove_file(&filepath) {
            Ok(..) => println!("+---- [INFO] removed file : {}", &filepath.display()),
            Err(error) => eprintln!("+---- [WARNING] failed to remove file : {}\n+---- ///////// {}", &filepath.display(), error),
        }
        println!("+-- [INFO] done")
    }
    pub fn default() -> io::Result<Self> {
        println!("+-- [INFO] creating default setvol config");
        let filepath: PathBuf = PathBuf::from(format!("{}_2.default", TEHRAN_NAME));
        if !filepath.is_file() {
            Self::default_install()?;
        };
        let mut buffer: String = String::new();
        let mut file = OpenOptions::new().read(true).open(&filepath)?;
        file.read_to_string(&mut buffer)?;
        let config = serde_json::from_str::<Self>(&buffer)?;
        println!("+-- [INFO] done");
        return Ok(config);
    }
    pub fn default_install() -> io::Result<()> {
        println!("+-- [INFO] downloading default setvol config...");
        download_file(None, DEFAULT_DOWNLOAD_LINK, &PathBuf::from(format!("{}_2.default", TEHRAN_NAME)))?;
        println!("+-- [INFO] done");
        return Ok(());
    }
}