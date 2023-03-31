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
    TEHRAN_NAME,
    soundlib::DEFAULT_DOWNLOAD_LINK,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SoundlibConfig {
    pub filenames: Vec<PathBuf>,
    pub path: PathBuf,
    pub archive_path: PathBuf,
    pub archive_filename: PathBuf,
    pub archive_hash: [u8; 32],
    pub archive_link: String,
}

impl SoundlibConfig {
    pub fn get_filepaths(&self) -> Vec<PathBuf> {
        let mut filepaths: Vec<PathBuf> = Vec::new();
        self.filenames.iter().for_each(|filename|{filepaths.push(self.path.join(filename))});
        return filepaths;
    }
    pub fn get_archive_filepath(&self) -> PathBuf {
        return self.archive_path.join(&self.archive_filename);
    }
    pub fn install(&self) {
        println!("+-- [INFO] installing soundlib...");
        match create_dir_all(&self.path) {
            Ok(..) => {
                let archive_filepath: PathBuf = self.get_archive_filepath();
                match extract_zip(&archive_filepath, &self.path) {
                    Ok(..) => {
                        println!("+-- [INFO] done -> {}", self.path.display());
                    },
                    Err(error) => {
                        eprintln!("+-- [WARNING] failed to unzip the soundlib backup archive : {} to {}\n+-- ///////// {}", archive_filepath.display(), self.path.display(), error);
                    }
                }
            }
            Err(error) => {
                eprintln!("+-- [WARNING] failed to create the soundlib directory : {}\n+-- /////// {}", self.path.display(), error);
            }
        }
    }
    pub fn uninstall(&self) {
        println!("+-- [INFO] uninstalling soundlib...");
        let filepaths: Vec<PathBuf> = self.get_filepaths();
        for filepath in filepaths.into_iter() {
            match remove_file(&filepath) {
                Ok(..) => println!("+---- [INFO] removed soundlib file"),
                Err(error) => eprintln!("+---- [WARNING] failed to remove soundlib file\n+---- ///////// {}",  error),
            }
        }
        match remove_dir(&self.path) {
            Ok(..) => println!("+---- [INFO] removed directory : {}", self.path.display()),
            Err(error) => eprintln!("+---- [WARNING] failed to remove directory : {}\n+---- ///////// {}", self.path.display(), error),
        };
    }
    pub fn archive_install(&self) {
        println!("+-- [INFO] downloading soundlib backup archive...");
        match download_file(Some(&self.archive_path), &self.archive_link, &self.archive_filename) {
            Ok(..) => println!("+-- [INFO] done"),
            Err(error) => eprintln!("+-- [WARNING] failed to download soundlib backup archive\n+-- ///////// {}", error)
        }
    }
    pub fn archive_uninstall(&self) {
        println!("+-- [INFO] removing soundlib backup archive...");
        let filepath: PathBuf = self.get_archive_filepath();
        match remove_file(&filepath) {
            Ok(..) => println!("+---- [INFO] removed file : {}", &filepath.display()),
            Err(error) => eprintln!("+---- [WARNING] failed to remove file : {}\n+---- ///////// {}", &filepath.display(), error),
        }
    }
    pub fn default() -> io::Result<Self> {
        println!("+-- [INFO] creating default soundlib config");
        let filepath: PathBuf = PathBuf::from(format!("{}_3.default", TEHRAN_NAME));
        if !filepath.is_file() {
            Self::default_install()?;
        };
        let mut buffer: String = String::new();
        let mut file = OpenOptions::new().read(true).open(&filepath)?;
        file.read_to_string(&mut buffer);
        let config = serde_json::from_str::<Self>(&buffer)?;
        println!("+-- [INFO] done");
        return Ok(config);
    }
    pub fn default_install() -> io::Result<()> {
        println!("+-- [INFO] downloading default soundlib config...");
        download_file(None, DEFAULT_DOWNLOAD_LINK, &PathBuf::from(format!("{}_3.default", TEHRAN_NAME)))?;
        println!("+-- [INFO] done");
        return Ok(());
    }
}