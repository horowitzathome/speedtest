use std::path::Path;
use std::time::Instant;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;

const BUFFER_SIZE: usize = 1024 * 1024; // 1 MB per chunk

pub async fn write_test_file<P: AsRef<Path>>(path: P, size_in_bytes: usize) -> Result<Duration, std::io::Error> {
    let mut file = File::create(&path).await?;
    let full_chunks = size_in_bytes / BUFFER_SIZE;
    let remainder = size_in_bytes % BUFFER_SIZE;

    let buffer = vec![0u8; BUFFER_SIZE];
    let start = Instant::now();

    // Write full chunks
    for _ in 0..full_chunks {
        file.write_all(&buffer).await?;
    }

    // Write remainder if needed
    if remainder > 0 {
        file.write_all(&buffer[..remainder]).await?;
    }

    file.flush().await?;
    Ok(start.elapsed())
}

pub async fn read_test_file<P: AsRef<Path>>(path: P) -> Result<Duration, std::io::Error> {
    let mut file = File::open(&path).await?;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let start = Instant::now();
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
    }

    Ok(start.elapsed())
}