extern crate secp256k1;
extern crate sha3;
extern crate rand;
extern crate hex;
extern crate ctrlc;
extern crate chrono;
extern crate clap;

use clap::Parser;
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufWriter, BufRead};
use secp256k1::{Secp256k1, PublicKey};
use sha3::{Digest, Keccak256};
use rand::rngs::OsRng;
use hex::encode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::rand::RngCore;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    in_file: String, // The 'in' argument renamed to 'in_file' to avoid keyword conflict
    #[clap(short, long)]
    out_file: String,
}

static DATE_FORMAT: &str = "%d-%m-%Y %H:%M:%S%.6f";

fn main() -> io::Result<()> {
    let args = Args::parse();
    
    // Read prefixes from the file specified by the 'in' argument
    let file = File::open(&args.in_file)?;
    let reader = io::BufReader::new(file);
    let wanted: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("Ctrl-C detected, stopping...");
    }).expect("Error setting Ctrl-C handler");

    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let path = format!(".\\{}", args.out_file);
    
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true) // Set to append mode
        .open(&path)?;
    let mut file = BufWriter::new(file);

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
    
        let now = Local::now();
        println!("{} {}:{}", now.format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes));
    
        for pattern in &wanted {
            if pattern.contains("...") {
                let parts: Vec<&str> = pattern.split("...").collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    if eth_address_display.starts_with(prefix) && eth_address_display.ends_with(suffix) {
                        writeln!(file, "{} {}:{}", now.format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");
                        file.flush().expect("Failed to flush output");
                        break; // Match found, no need to check further
                    }
                }
            } else if eth_address_display.starts_with(pattern) {
                writeln!(file, "{} {}:{}", now.format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");
                file.flush().expect("Failed to flush output");
                break; // Match found, no need to check further
            }
        }
    }
    Ok(())
}