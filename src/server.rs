use num_format::Locale;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::Duration;

use crate::{
    utils::{format_number, print_statistics_terminal}, Direction
};

pub async fn run_server(port: u16, block_size_kb: usize, default_duration_secs: u64) {
    let listener = TcpListener::bind(("0.0.0.0", port)).await.expect("Failed to bind");
    println!("Server listening on port {} ...", port);

    let total_bytes = Arc::new(AtomicU64::new(0));
    let total_duration = Arc::new(Mutex::new(Duration::ZERO));
    let clients = Arc::new(Mutex::new(0usize));
    let (quit_tx, mut quit_rx) = tokio::sync::watch::channel(false);

    loop {
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                println!("Accepted connection from {}", addr);

                let bytes = Arc::clone(&total_bytes);
                let clients = Arc::clone(&clients);
                let total_duration = Arc::clone(&total_duration);
                let quit_signal = quit_tx.clone();

                tokio::spawn(async move {
                    let mut reader = BufReader::new(socket);
                    let mut mode_line = String::new();
                    let mut duration_line = String::new();

                    if reader.read_line(&mut mode_line).await.unwrap_or(0) == 0 {
                        eprintln!("Failed to read mode from client {}", addr);
                        return;
                    }

                    let mode_str = mode_line.trim();
                    let mode = match mode_str {
                        "upload" => Direction::Upload,
                        "download" => Direction::Download,
                        "bidirectional" => Direction::Bidirectional,
                        "quit" => Direction::Quit,
                        _ => {
                            eprintln!("Unknown direction '{}' from {}", mode_str, addr);
                            return;
                        }
                    };

                    if mode == Direction::Quit {
                        println!("Quit signal received from {}", addr);
                        let _ = quit_signal.send(true);
                        return;
                    }

                    let mut duration_secs = default_duration_secs;
                    if matches!(mode, Direction::Download | Direction::Bidirectional) {
                        if reader.read_line(&mut duration_line).await.unwrap_or(0) == 0 {
                            eprintln!("Expected duration line from client {}", addr);
                            return;
                        }

                        match duration_line.trim().parse::<u64>() {
                            Ok(secs) => duration_secs = secs,
                            Err(_) => {
                                eprintln!("Invalid duration from {}: '{}'", addr, duration_line.trim());
                                return;
                            }
                        }
                    }

                    let mut socket = reader.into_inner();

                    let mut buf = vec![0u8; block_size_kb * 1024];
                    let mut local_bytes = 0u64;
                    let deadline = Instant::now() + Duration::from_secs(duration_secs);
                    let start = Instant::now();

                    match mode {
                        Direction::Upload => {
                            while let Ok(n) = socket.read(&mut buf).await {
                                if n == 0 { break; }
                                local_bytes += n as u64;
                            }
                        }
                        Direction::Download => {
                            while Instant::now() < deadline {
                                if socket.write_all(&buf).await.is_err() { break; }
                                local_bytes += buf.len() as u64;
                            }
                        }
                        Direction::Bidirectional => {
                            while Instant::now() < deadline {
                                if socket.write_all(&buf).await.is_err() { break; }
                                local_bytes += buf.len() as u64;
                                if let Ok(n) = socket.read(&mut buf).await {
                                    if n == 0 { break; }
                                    local_bytes += n as u64;
                                }
                            }
                        }
                        _ => {}
                    }

                    {
                        let duration = start.elapsed();
                        let mut total_duration = total_duration.lock().await;
                        *total_duration += duration;
                    }

                    bytes.fetch_add(local_bytes, Ordering::Relaxed);
                    println!(
                        "Client {} disconnected ({} MB)",
                        addr,
                        format_number(local_bytes as f64 / 1_000_000.0, &Locale::de)
                    );

                    let mut count = clients.lock().await;
                    *count += 1;
                });
            }

            changed = quit_rx.changed() => {
                if changed.is_ok() && *quit_rx.borrow() {
                    println!("Shutdown signal received. Exiting server loop.");
                    break;
                }
            }
        }
    }

    // Statistic
    let total = total_bytes.load(Ordering::Relaxed);
    let duration = total_duration.lock().await.as_secs_f64();

    println!("\n[ERGEBNIS]");
    print_statistics_terminal(duration, total);
}
