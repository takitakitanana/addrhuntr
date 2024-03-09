extern crate secp256k1;
extern crate sha3;
extern crate rand;
extern crate hex;
extern crate ctrlc;
extern crate num_cpus;
extern crate chrono; // Add this at the beginning of your file

use clap::Parser;
use chrono::Local; // Add this to your imports
use std::fs::OpenOptions;
use std::io::Write;
use secp256k1::{Secp256k1, PublicKey};
use sha3::{Digest, Keccak256};
use rand::{rngs::OsRng, RngCore};
use hex::encode;
use std::{sync::Arc, sync::atomic::{AtomicBool, Ordering}, thread};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    file: String,

    #[clap(short, long, default_value = "1")]
    cores: usize,
}


static WANTED: &[&str] = &[
    //"0xdead", // assuming '0x' prefix for consistency
    "0x0000000",
    "0x00000000",
    "0x000000000",
    "0x0000000000",
    "0x0123456789",
];

static DATE_FORMAT: &str = "%d-%M-%Y %H:%M:%S";

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Clone file_path to use it in the closure
    let file_path = args.file.clone(); 

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("Bye.");
    }).expect("Error setting Ctrl-C handler");

    let num_threads = 4;
    //let num_threads = num_cpus::get();
    let mut handles = vec![];

    for _ in 0..num_threads {
        let running = running.clone();
        let file_path = file_path.clone(); // Clone file_path for each iteration

        let handle = thread::spawn(move || {
            let secp = Secp256k1::new();
            let mut rng = OsRng;

            while running.load(Ordering::SeqCst) {
                let mut private_key_bytes = [0u8; 32];
                rng.fill_bytes(&mut private_key_bytes);
                let private_key = secp256k1::SecretKey::from_slice(&private_key_bytes)
                    .expect("32 bytes, within curve order");
                let public_key = PublicKey::from_secret_key(&secp, &private_key);
                let serialized_pub = public_key.serialize_uncompressed();
                let mut hasher = Keccak256::new();
                hasher.update(&serialized_pub[1..]);
                let hashed_pub = hasher.finalize();
                let eth_address = &hashed_pub[hashed_pub.len() - 20..];
                let eth_address_hex = encode(eth_address);
                let eth_address_display = format!("0x{}", eth_address_hex);

                // Check if the Ethereum address starts with any of the specified prefixes
                for prefix in WANTED {
                    if eth_address_display.starts_with(prefix) {
                        let now = Local::now(); // Get the current local time and date
                        let mut file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&file_path)
                            .expect("Unable to open file");

                        writeln!(file, "===============================================================================================================================").expect("Unable to write to file");
                        writeln!(file, "{} {}:{}", now.format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");

                        println!("===============================================================================================================================");
                        println!("{} {}:{}", now.format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes));

                        // Optionally, break out of the loop if you only care about the first match
                        // break;
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
