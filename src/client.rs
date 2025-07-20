use crate::{
    Direction,
    file::{read_test_file, write_test_file},
    utils::{generate_test_sizes, print_statistics},
};
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::sleep,
};

pub async fn run_client_loop(address: String, threads: usize, block_size_kb: usize, duration_secs: u64, path: &str, file_size_mb: usize) {
    // Then do file write and read test
    let sizes = generate_test_sizes(file_size_mb);

    loop {
        // First do HTTP test with download and upload
        run_client(address.clone(), threads, block_size_kb, duration_secs, Direction::Download).await;
        sleep(Duration::from_secs(1)).await;
        run_client(address.clone(), threads, block_size_kb, duration_secs, Direction::Upload).await;
        sleep(Duration::from_secs(1)).await;

        for size in &sizes {
            // First write file
            let duration_res = write_test_file(path, *size).await.unwrap();
            print_statistics(duration_res.as_secs_f64(), *size, Direction::Upload, 0, path);

            // Now read file
            let duration_res = read_test_file(path).await.unwrap();
            print_statistics(duration_res.as_secs_f64(), *size, Direction::Download, 0, path);
        }
    }
}

pub async fn run_client(address: String, threads: usize, block_size_kb: usize, duration_secs: u64, direction: Direction) {
    println!("Connecting to {} with {} async tasks in '{:?}' mode", address, threads, direction);
    let total_bytes = Arc::new(AtomicUsize::new(0));
    let block_size = block_size_kb * 1024;

    let mut handles = Vec::new();
    for _ in 0..threads {
        let addr = address.clone();
        let bytes = Arc::clone(&total_bytes);
        let dir = direction;

        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(&addr).await.expect("Failed to connect");
            let mode = match direction {
                Direction::Download => format!("download\n{}\n", duration_secs),
                Direction::Upload => "upload\n".to_string(),
                Direction::Bidirectional => format!("bidirectional\n{}\n", duration_secs),
                Direction::Quit => "quit\n".to_string(),
            };
            stream.write_all(mode.as_bytes()).await.expect("Failed to send direction");

            let mut buf = vec![0u8; block_size];
            let start = Instant::now();
            let deadline = start + Duration::from_secs(duration_secs);

            //println!("start / end: {}", format_duration_hms(start, deadline));

            let mut count = 0;

            match dir {
                Direction::Upload => {
                    while Instant::now() < deadline {
                        if stream.write_all(&buf).await.is_err() {
                            break;
                        }
                        count += buf.len();
                    }
                }
                Direction::Download => {
                    while Instant::now() < deadline {
                        match stream.read(&mut buf).await {
                            Ok(0) => break,
                            Ok(n) => count += n,
                            Err(_) => break,
                        }
                    }
                }
                Direction::Bidirectional => {
                    // Optional: alternate sending and receiving in bidirectional mode
                    while Instant::now() < deadline {
                        if stream.write_all(&buf).await.is_err() {
                            break;
                        }
                        count += buf.len();
                        if let Ok(n) = stream.read(&mut buf).await {
                            if n == 0 {
                                break;
                            }
                            count += n;
                        }
                    }
                }
                Direction::Quit => { /* Do nothing, is for server only */ }
            }

            bytes.fetch_add(count, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    let duration = duration_secs as f64;
    let total = total_bytes.load(Ordering::Relaxed);

    println!("\n[ERGEBNIS]");
    println!("Richtung: {:?}", direction);
    print_statistics(duration, total, direction, block_size_kb, &address);
}
