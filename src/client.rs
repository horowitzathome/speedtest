use crate::utils::format_number;
use num_format::Locale;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};

pub async fn run_client(address: String, duration_secs: u64, threads: usize, block_size_factor: usize) {
    println!("Connecting to {} with {} async tasks", address, threads);
    let total_bytes = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    for _ in 0..threads {
        let addr = address.clone();
        let bytes = Arc::clone(&total_bytes);

        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(&addr).await.expect("Failed to connect");
            let buf = vec![0u8; block_size_factor * 1024];
            let start = Instant::now();
            let deadline = start + Duration::from_secs(duration_secs);

            let mut sent = 0u64;
            while Instant::now() < deadline {
                match stream.write_all(&buf).await {
                    Ok(_) => sent += buf.len() as u64,
                    Err(e) => {
                        eprintln!("Write error: {:?}", e);
                        break;
                    }
                }
            }

            bytes.fetch_add(sent, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    let duration = duration_secs as f64;
    let total = total_bytes.load(Ordering::Relaxed);

    let locale = Locale::de;

    // Sizes
    let total_mbytes = total as f64 / 1_000_000.0;
    let total_mbits = total_mbytes * 8.0;
    let total_gbytes = total_mbytes / 1_000.0;
    let total_gbits = total_mbits / 1_000.0;

    // Speeds
    let kbytes_per_sec = (total as f64 / 1_000.0) / duration;
    let kbits_per_sec = kbytes_per_sec * 8.0;
    let mbytes_per_sec = total_mbytes / duration;
    let mbits_per_sec = total_mbits / duration;
    let gbytes_per_sec = total_gbytes / duration;
    let gbits_per_sec = total_gbits / duration;

    let minutes = (duration as u64) / 60;
    let seconds = (duration as u64) % 60;

    println!("\n[ERGEBNIS]");

    println!("• Testdauer:");
    println!("   - {} min {} s", minutes, seconds);
    println!("   - {} Sekunden", format_number(duration, &locale));

    println!("• Gesendet:");
    println!("   - {} MByte", format_number(total_mbytes, &locale));
    println!("   - {} MBit", format_number(total_mbits, &locale));
    println!("   - {} GByte", format_number(total_gbytes, &locale));
    println!("   - {} GBit", format_number(total_gbits, &locale));

    println!("• Durchsatz:");
    println!("   - {} KByte/s", format_number(kbytes_per_sec, &locale));
    println!("   - {} KBit/s", format_number(kbits_per_sec, &locale));
    println!("   - {} MByte/s", format_number(mbytes_per_sec, &locale));
    println!("   - {} MBit/s", format_number(mbits_per_sec, &locale));
    println!("   - {} GByte/s", format_number(gbytes_per_sec, &locale));
    println!("   - {} GBit/s", format_number(gbits_per_sec, &locale));
}
