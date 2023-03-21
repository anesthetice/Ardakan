use rsa::{RsaPrivateKey, Pkcs1v15Encrypt, pkcs8::DecodePrivateKey};
use lazy_static::{lazy_static, initialize as ls_initialize};
use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, Nonce, KeyInit};
use std::{
    net::TcpStream,
    io::{
        self,
        Read,
        Write,
    },
};

mod constants;
use constants::PEM_CLIENT_PRIVATE_KEY;


lazy_static! {
    pub static ref CLIENT_PRIVATE_KEY : RsaPrivateKey = RsaPrivateKey::from_pkcs8_pem(&PEM_CLIENT_PRIVATE_KEY).unwrap();
}

fn main() -> std::io::Result<()> {
    let print_help = || {
        println!("Ardakan-Client version 1.0");
        println!("Usage: ardakan-client [IP_ADDRESS]:[PORT]");
        println!("Options:");
        println!("  -h, --help    print help");
        println!("Instructions:");
        println!("  cmd, bat         flags = (wait, repeat, explicit)");
        println!("  website          flags = (wait, repeat)");
        println!("  sound            flags = (wait, repeat, time)");
        println!("  soundlib         flags = (wait, repeat, time)");
        println!("  echo             flags = ( wait, repeat)");
        println!("  exit");
        println!("Flags:");
        println!("  -t, --time");
        println!("  -w, --wait");
        println!("  -r, --repeat");
        println!("  -e, --explicit");
        println!("Meta-Instructions");
        println!("  REPEAT () ... /REPEAT ()");
    };

    ls_initialize(&CLIENT_PRIVATE_KEY);

    let ip_address : String = match std::env::args().nth(1) {
        Some(string) => {
            if string.contains("--help") || string.contains("-h") {
                print_help();
                return Ok(())
            } else {
                string
            }
        },
        None => {
            print_help();
            return Ok(())
        },
    };
    
    println!("[INFO] attempting to connect to {}...", &ip_address);
    let mut stream : TcpStream = TcpStream::connect(&ip_address)?;
    
    // RSA authentication
    {
        let mut buffer : [u8; 256] = [0; 256];
        stream.read(&mut buffer)?;
        let token : Vec<u8> = CLIENT_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &buffer).unwrap();
        stream.write(&token)?;
    }

    // AES encryption
    let mut buffer : [u8; 256] = [0; 256];
    stream.read(&mut buffer)?;
    let key : Vec<u8> = CLIENT_PRIVATE_KEY.decrypt(Pkcs1v15Encrypt, &buffer).unwrap();
    let cipher = Aes256GcmSiv::new_from_slice(&key[..]).unwrap();
    let nonce = Nonce::from_slice(&key[..12]);
    println!("[INFO] connection secured");

    loop {

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.starts_with("help") {
            print_help();
            continue;
        }

        let mut ciphertext = cipher.encrypt(nonce, input.as_bytes().as_ref()).unwrap();
        let mut ciphertext_length = ciphertext.len();

        if ciphertext_length <= 2040 {
            let mut counter : u8 = 0;
            while ciphertext_length != 0 {
                if ciphertext.len() > 255 {
                    ciphertext.insert(0, 255);
                    counter += 1;
                    ciphertext_length -= 255;
                } else {
                    ciphertext.insert(0, ciphertext_length as u8);
                    counter += 1;
                    ciphertext_length = 0;
                }
            }
            while counter != 8 {
                ciphertext.insert(0, 0);
                counter += 1;
            }
            stream.write(&ciphertext)?;
        } else {
            println!("[ERROR] message cannot exceed 1020 bytes")
        }

        if input.starts_with("exit") || input.ends_with(";; exit") || input.contains(";; exit ;;"){
            break;
        }
    }

    return Ok(());
}
