use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub async fn run_server(port: u16, expected_clients: usize, block_size_factor: usize) {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to bind");
    println!(
        "Server listening on port {} for {} clients...",
        port, expected_clients
    );

    let total_bytes = Arc::new(AtomicU64::new(0));
    let clients = Arc::new(Mutex::new(0usize));
    let start = Instant::now();

    loop {
        let (mut socket, addr) = listener.accept().await.expect("Failed to accept");
        println!("Accepted connection from {}", addr);

        let bytes = Arc::clone(&total_bytes);
        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            let mut buf = vec![0u8; block_size_factor * 1024];
            let mut local_bytes = 0u64;

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => local_bytes += n as u64,
                    Err(e) => {
                        eprintln!("Read error: {:?}", e);
                        break;
                    }
                }
            }

            bytes.fetch_add(local_bytes, Ordering::Relaxed);
            println!(
                "Client {} disconnected ({} MB)",
                addr,
                local_bytes as f64 / 1_000_000.0
            );

            let mut count = clients.lock().await;
            *count += 1;

            if *count >= expected_clients {
                let duration = start.elapsed().as_secs_f64();
                let total = bytes.load(Ordering::Relaxed);
                let mbps = (total as f64 * 8.0) / (duration * 1_000_000.0);
                println!(
                    "\n[FINAL] Total received: {:.2} MB in {:.2} seconds ({:.2} Mbps)",
                    total as f64 / 1_000_000.0,
                    duration,
                    mbps
                );
                //std::process::exit(0);
            }
        });
    }
}
