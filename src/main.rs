extern crate secp256k1;
extern crate sha3;
extern crate rand;
extern crate hex;
extern crate ctrlc;
extern crate chrono;
extern crate clap;
extern crate num_format;

use clap::Parser;
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
use serde_json;

#[derive(Parser)]
#[clap(author = "takitakitanana", version = "2.2", about = "Hunting specific addresses.", long_about = None)]
struct Args {
    /// Input file
    #[clap(short, long, help = "Input file (optional).", default_value = "/wallets/find.txt")]
    //#[clap(short, long, help = "Input file")]
    in_file: String,

    /// Output file
    #[clap(short, long, help = "Output file (optional).", default_value = "/wallets/found.txt")]
    //#[clap(short, long, help = "Output file")]
    out_file: String,

    /// Discord webhook URL argument
    #[clap(short = 'd', long = "discord", help = "Discord Webhook URL (optional).")]
    discord_webhook_url: Option<String>,

    /// User mention argument
    #[clap(short = 'u', long = "user", help = "User ID to mention in Discord message (optional).")]
    user_mention: Option<String>,

    /// Ping
    #[clap(short, long, help = "Ping for address prefix.", default_value = "0x00000000")]
    ping: String,
    }

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let file = File::open(&args.in_file)?;
    let reader = io::BufReader::new(file);
    let wanted: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nBye.");
    }).expect("Error setting Ctrl-C handler");

    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let path = format!("{}", args.out_file);

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
                        println!("{}", eth_address_display);
                        writeln!(file, "{} {}", eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");
                        file.flush().expect("Failed to flush output");
                        break;
                    }
                }
            } else if eth_address_display.starts_with(pattern) {

                // Assuming `eth_address_display` starts with "0xdead"
                if eth_address_display.starts_with(&args.ping) {
                    // Check if both the Discord webhook URL and user mention were provided
                    if let (Some(webhook_url), Some(user_mention)) = (&args.discord_webhook_url, &args.user_mention) {
                        // Construct the message payload with user mention
                        let payload = serde_json::json!({
                            "content": format!("{} found {}", user_mention, eth_address_display)
                        });

                        // Send the Discord message
                        let client = reqwest::Client::new();
                        match client.post(webhook_url)
                            .json(&payload)
                            .send()
                            .await {
                                Ok(_) => println!("-> notification sent to Discord."),
                                Err(e) => println!("-> failed to send Discord notification: {}", e),
                        }
                    }
                }

            let mut found = found_addresses.lock().unwrap();
            *found += 1;
            println!("{}", eth_address_display);
            writeln!(file, "{} {}", eth_address_display, encode(&private_key_bytes)).expect("Unable to write to file");
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
        format!("{:.2}tb", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2}gb", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}mb", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}kb", bytes as f64 / KB as f64)
    } else {
        format!("{}b", bytes)
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

    let bytes_per_found = 107;
    let filesize_increase_per_minute = avg_found_per_minute * bytes_per_found;

    print!("\r\x1B[K| uptime {} | found {} | avg/min {} | size/min {} | ",
        runtime_formatted,
        //addresses_generated.to_formatted_string(&Locale::en),
        found.to_formatted_string(&Locale::en),
        avg_found_per_minute.to_formatted_string(&Locale::en),
        format_bytes(filesize_increase_per_minute),
    );
    io::stdout().flush()?;
    *last_update = Instant::now();

    Ok(())
}
