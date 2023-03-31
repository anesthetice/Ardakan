#![allow(non_snake_case)]

use serde::{
    Deserialize,
    Serialize
};
use std::{
    io::{Read, Write, self},
    fs::{File, OpenOptions},
};
use crate::{
    ardakan_config::ArdakanConfig,
    setvol_config::SetvolConfig,
    soundlib_config::SoundlibConfig,
    utils::{replace_homedrive, replace_username},
    constants::TEHRAN_NAME
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub ardakan: ArdakanConfig,
    pub setvol: SetvolConfig,
    pub soundlib: SoundlibConfig, 
}

impl Configuration {
    pub fn load(HOMEDRIVE: &str, USERNAME: &str) -> io::Result<Self> {
        println!("+-- [INFO] loading configuration...");
        let mut buffer: String = String::new();
        let mut file: File = OpenOptions::new().read(true).open(&format!("{}.config", TEHRAN_NAME))?;
        file.read_to_string(&mut buffer)?;
        let mut configuration: Self = serde_json::from_str(&buffer)?;
        configuration.replace_variables(HOMEDRIVE, USERNAME);
        println!("+-- [INFO] done");
        return Ok(configuration);
    }

    pub fn save(&self) -> io::Result<()> {
        println!("+-- [INFO] saving configuration...");
        let mut file: File = OpenOptions::new().create(true).write(true).truncate(true).open(&format!("{}.config", TEHRAN_NAME))?;
        file.write_all(serde_json::to_string(self)?.as_bytes())?;
        println!("+-- [INFO] done");
        return Ok(());
    }

    pub fn default(HOMEDRIVE: &str, USERNAME: &str) -> io::Result<Self> {
        println!("+-- [INFO] creating default configuration...");
        let mut configuration: Self = Self {
            ardakan: ArdakanConfig::default(HOMEDRIVE, USERNAME)?, 
            setvol: SetvolConfig::default()?, 
            soundlib: SoundlibConfig::default()?,
        };
        configuration.replace_variables(HOMEDRIVE, USERNAME);
        println!("+-- [INFO] done");
        return Ok(configuration);
    }
    
    pub fn replace_variables(&mut self, HOMEDRIVE: &str, USERNAME: &str) {
        replace_homedrive(&mut self.ardakan.path, HOMEDRIVE); replace_homedrive(&mut self.ardakan.archive_path, HOMEDRIVE); 
        replace_username(&mut self.ardakan.path, USERNAME); replace_username(&mut self.ardakan.archive_path, USERNAME);
        replace_homedrive(&mut self.setvol.path, HOMEDRIVE); replace_homedrive(&mut self.setvol.archive_path, HOMEDRIVE); 
        replace_username(&mut self.setvol.path, USERNAME); replace_username(&mut self.setvol.archive_path, USERNAME);
        replace_homedrive(&mut self.soundlib.path, HOMEDRIVE); replace_homedrive(&mut self.soundlib.archive_path, HOMEDRIVE); 
        replace_username(&mut self.soundlib.path, USERNAME); replace_username(&mut self.soundlib.archive_path, USERNAME);
    }
}
