use windows::{
    Win32::Foundation::*, 
    Win32::System::SystemServices::*,
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
            use std::process::Command;
            Command::new("cmd").args(["/C", "cd /D C:\\ && echo hello>test.txt"]).spawn();
        },
        DLL_PROCESS_DETACH => (),
        _ => ()
    }
    true
}


use sha2::{
    Digest,
    Sha256,
};

use std::{
    fs,
    io,
};

fn hash_file() {
    let path = "test.txt";
    let mut file = fs::File::open(&path).unwrap();
    let mut hasher = Sha256::new();
    let n = io::copy(&mut file, &mut hasher);
    let hash = hasher.finalize();
    println!("{:?}", hash);
}
