
/*
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
        },
        DLL_PROCESS_DETACH => (),
        _ => ()
    }
    true
}
*/

use sha2::{
    Digest,
    Sha256,
};

use std::{
    fs::OpenOptions,
    io,
};

fn hash_file(filepath : &str) -> Result<&[u8], ()> {
    let mut file = match OpenOptions::new().read(true).open(filepath) {
        Ok(file) => file,
        Err(_) => return Err(()),
    };
    let mut hasher = Sha256::new();
    match io::copy(&mut file, &mut hasher) {
        Ok(..) => (),
        Err(_) => return Err(()),   
    }    
    let hash : &[u8] = hasher.finalize().as_ref();
    Ok(hash)
}

fn main() {
}