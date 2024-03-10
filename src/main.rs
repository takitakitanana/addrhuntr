extern crate secp256k1;
extern crate sha3;
extern crate rand;
extern crate hex;
extern crate ctrlc;
extern crate chrono;
extern crate clap;
extern crate num_format;

use clap::Parser;
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufWriter, BufRead};
use secp256k1::{Secp256k1, PublicKey};
use sha3::{Digest, Keccak256};
use rand::rngs::OsRng;
use hex::encode;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, Duration};
use num_format::{Locale, ToFormattedString};
use rand::RngCore;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    in_file: String,
    #[clap(short, long)]
    out_file: String,
}

static DATE_FORMAT: &str = "%d-%m-%Y %H:%M:%S%.6f";

fn main() -> io::Result<()> {
    let args = Args::parse();

    let file = File::open(&args.in_file)?;
    let reader = io::BufReader::new(file);
    let wanted: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nCtrl-C detected, stopping...");
    }).expect("Error setting Ctrl-C handler");

    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let path = format!(".\\{}", args.out_file);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)?;
    let mut file = BufWriter::new(file);

    let start_time = Instant::now();
    let mut last_update = start_time;
    let mut addresses_generated: u64 = 0;
    let found_addresses = Arc::new(Mutex::new(0u64));

    // Initial update_statistics call - modified to include found_addresses
    update_statistics(start_time, &mut last_update, addresses_generated, found_addresses.clone())?;

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

        addresses_generated += 1;

        if Instant::now() - last_update >= Duration::from_secs(1) {
            // Updated to include found_addresses in statistics update
            update_statistics(start_time, &mut last_update, addresses_generated, found_addresses.clone())?;
        }

        for pattern in &wanted {
            if pattern.contains("...") {
                let parts: Vec<&str> = pattern.split("...").collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    if eth_address_display.starts_with(prefix) && eth_address_display.ends_with(suffix) {
                        let mut found = found_addresses.lock().unwrap();
                        *found += 1;
                        println!("\n{} {}", Local::now().format(DATE_FORMAT), eth_address_display);
                        writeln!(file, "{} {}:{}", Local::now().format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");
                        file.flush().expect("Failed to flush output");
                        break;
                    }
                }
            } else if eth_address_display.starts_with(pattern) {
                let mut found = found_addresses.lock().unwrap();
                *found += 1;
                println!("\n{} {}", Local::now().format(DATE_FORMAT), eth_address_display);
                writeln!(file, "{} {}:{}", Local::now().format(DATE_FORMAT), eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");
                file.flush().expect("Failed to flush output");
                break;
            }
        }
    }

    Ok(())
}

fn format_duration(runtime_seconds: u64) -> String {
    let seconds = runtime_seconds % 60;
    let minutes = (runtime_seconds / 60) % 60;
    let hours = (runtime_seconds / (60 * 60)) % 24;
    let days = runtime_seconds / (60 * 60 * 24);

    let mut duration_str = String::new();
    if days > 0 {
        duration_str.push_str(&format!("{}d ", days));
    }
    if hours > 0 {
        duration_str.push_str(&format!("{:02}h ", hours));
    }
    if minutes > 0 {
        duration_str.push_str(&format!("{:02}m ", minutes));
    }
    duration_str.push_str(&format!("{:02}s", seconds));

    duration_str // Ensure this line is present to return the constructed string
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} tb", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} gb", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} mb", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} kb", bytes as f64 / KB as f64)
    } else {
        format!("{} b", bytes)
    }
}


fn update_statistics(start_time: Instant, last_update: &mut Instant, addresses_generated: u64, found_addresses: Arc<Mutex<u64>>) -> io::Result<()> {
    let runtime_seconds = Instant::now().duration_since(start_time).as_secs();
    let avg_per_minute = if runtime_seconds > 0 {
        addresses_generated * 60 / runtime_seconds
    } else {
        0
    };
    let avg_per_hour = avg_per_minute * 60;
    let avg_per_day = avg_per_hour * 24;
    let avg_per_month = avg_per_day * 30; // Approximation for generated addresses

    let runtime_formatted = format_duration(runtime_seconds);

    let found = *found_addresses.lock().unwrap();
    let avg_found_per_minute = if runtime_seconds > 0 && found > 0 {
        found * 60 / runtime_seconds
    } else {
        0
    };
    let avg_found_per_hour = avg_found_per_minute * 60;
    let avg_found_per_day = avg_found_per_hour * 24;
    let avg_found_per_month = avg_found_per_day * 30; // Approximation for found addresses

    // File size estimation: one entry = 134 bytes
    let bytes_per_found = 134;
    let filesize_increase_per_minute = avg_found_per_minute * bytes_per_found;
    let filesize_increase_per_hour = avg_found_per_hour * bytes_per_found;
    let filesize_increase_per_day = avg_found_per_day * bytes_per_found;
    let filesize_increase_per_month = avg_found_per_month * bytes_per_found;

    // Use format_bytes for displaying filesize increases in a readable format
    print!("\r\x1B[Kuptime {} | addr {} / {} | avg/m {} / {} ({}) | avg/h {} / {} ({}) | avg/d {} / {} ({}) | avg/m {} / {} ({})",
        runtime_formatted,
        addresses_generated.to_formatted_string(&Locale::en),
        found.to_formatted_string(&Locale::en),
        avg_per_minute.to_formatted_string(&Locale::en),
        avg_found_per_minute.to_formatted_string(&Locale::en),
        format_bytes(filesize_increase_per_minute),
        avg_per_hour.to_formatted_string(&Locale::en),
        avg_found_per_hour.to_formatted_string(&Locale::en),
        format_bytes(filesize_increase_per_hour),
        avg_per_day.to_formatted_string(&Locale::en),
        avg_found_per_day.to_formatted_string(&Locale::en),
        format_bytes(filesize_increase_per_day),
        avg_per_month.to_formatted_string(&Locale::en),
        avg_found_per_month.to_formatted_string(&Locale::en),
        format_bytes(filesize_increase_per_month),
    );
    io::stdout().flush()?;
    *last_update = Instant::now();

    Ok(())
}