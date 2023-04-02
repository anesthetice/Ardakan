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
    log,
    ardakan_config::ArdakanConfig,
    setvol_config::SetvolConfig,
    soundlib_config::SoundlibConfig,
    utils::replace_homedrive,
    constants::TEHRAN_NAME
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub ardakan: ArdakanConfig,
    pub setvol: SetvolConfig,
    pub soundlib: SoundlibConfig, 
}

impl Configuration {
    pub fn load(HOMEDRIVE: &str, logging: bool) -> io::Result<Self> {
        if logging{log("+-- [INFO] loading configuration...")};
        let mut buffer: String = String::new();
        let mut file: File = OpenOptions::new().read(true).open(&format!("{}.config", TEHRAN_NAME))?;
        file.read_to_string(&mut buffer)?;
        let mut configuration: Self = serde_json::from_str(&buffer)?;
        configuration.replace_variables(HOMEDRIVE);
        if logging{log("+-- [INFO] done")};
        return Ok(configuration);
    }

    pub fn save(&self) -> io::Result<()> {
        log("+-- [INFO] saving configuration...");
        let mut file: File = OpenOptions::new().create(true).write(true).truncate(true).open(&format!("{}.config", TEHRAN_NAME))?;
        file.write_all(serde_json::to_string(self)?.as_bytes())?;
        log("+-- [INFO] done");
        return Ok(());
    }

    pub fn default(HOMEDRIVE: &str) -> io::Result<Self> {
        log("+-- [INFO] creating default configuration...");
        let mut configuration: Self = Self {
            ardakan: ArdakanConfig::default(HOMEDRIVE)?, 
            setvol: SetvolConfig::default()?, 
            soundlib: SoundlibConfig::default()?,
        };
        configuration.replace_variables(HOMEDRIVE);
        log("+-- [INFO] done");
        return Ok(configuration);
    }
    
    pub fn replace_variables(&mut self, HOMEDRIVE: &str) {
        replace_homedrive(&mut self.ardakan.path, HOMEDRIVE); replace_homedrive(&mut self.ardakan.archive_path, HOMEDRIVE); 
        replace_homedrive(&mut self.setvol.path, HOMEDRIVE); replace_homedrive(&mut self.setvol.archive_path, HOMEDRIVE); 
        replace_homedrive(&mut self.soundlib.path, HOMEDRIVE); replace_homedrive(&mut self.soundlib.archive_path, HOMEDRIVE); 
    }
}
