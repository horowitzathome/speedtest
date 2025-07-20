use crate::Direction;
use chrono::Local;
use num_format::{Locale, ToFormattedString};
use std::io::Write;
use std::{fs::OpenOptions, io::BufWriter, path::Path};
use tokio::time::Instant;

pub struct Statistics {
    pub duration: f64,
    pub total_bytes: usize,
    pub total_mbytes: f64,
    pub total_mbits: f64,
    pub total_gbytes: f64,
    pub total_gbits: f64,
    pub kbytes_per_sec: f64,
    pub kbits_per_sec: f64,
    pub mbytes_per_sec: f64,
    pub mbits_per_sec: f64,
    pub gbytes_per_sec: f64,
    pub gbits_per_sec: f64,
    pub minutes: u64,
    pub seconds: u64,
}

fn calculate_statistics(duration: f64, total_bytes: usize) -> Statistics {
    let total_mbytes = total_bytes as f64 / 1_000_000.0;
    let total_mbits = total_mbytes * 8.0;
    let total_gbytes = total_mbytes / 1_000.0;
    let total_gbits = total_mbits / 1_000.0;

    let kbytes_per_sec = (total_bytes as f64 / 1_000.0) / duration;
    let kbits_per_sec = kbytes_per_sec * 8.0;
    let mbytes_per_sec = total_mbytes / duration;
    let mbits_per_sec = total_mbits / duration;
    let gbytes_per_sec = total_gbytes / duration;
    let gbits_per_sec = total_gbits / duration;

    let minutes = (duration as u64) / 60;
    let seconds = (duration as u64) % 60;

    Statistics {
        duration,
        total_bytes,
        total_mbytes,
        total_mbits,
        total_gbytes,
        total_gbits,
        kbytes_per_sec,
        kbits_per_sec,
        mbytes_per_sec,
        mbits_per_sec,
        gbytes_per_sec,
        gbits_per_sec,
        minutes,
        seconds,
    }
}

fn write_statistics_terminal(stats: &Statistics) {
    let locale = Locale::de;

    println!("• Dauer:");
    println!("   - {} min {} s", stats.minutes, stats.seconds);
    println!("   - {} Sekunden", format_number(stats.duration, &locale));

    println!("• Übertragen");
    println!("   - {} MByte", format_number(stats.total_mbytes, &locale));
    println!("   - {} MBit", format_number(stats.total_mbits, &locale));
    println!("   - {} GByte", format_number(stats.total_gbytes, &locale));
    println!("   - {} GBit", format_number(stats.total_gbits, &locale));

    println!("• Durchsatz:");
    println!("   - {} KByte/s", format_number(stats.kbytes_per_sec, &locale));
    println!("   - {} KBit/s", format_number(stats.kbits_per_sec, &locale));
    println!("   - {} MByte/s", format_number(stats.mbytes_per_sec, &locale));
    println!("   - {} MBit/s", format_number(stats.mbits_per_sec, &locale));
    println!("   - {} GByte/s", format_number(stats.gbytes_per_sec, &locale));
    println!("   - {} GBit/s", format_number(stats.gbits_per_sec, &locale));
}

fn write_statistics_csv(stats: &Statistics, direction: Direction, block_size_kb: usize, remote_addr: &str) {
    let locale = Locale::de;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let csv_path = Path::new("results.csv");
    let file_exists = csv_path.exists();

    let file = OpenOptions::new().append(true).create(true).open(csv_path).expect("Unable to open or create CSV file");
    let mut writer = BufWriter::new(file);

    if !file_exists {
        writeln!(
            writer,
            "Zeitpunkt;Adresse;Richtung;Blockgröße (KB);Dauer (s);Gesamt MByte;Gesamt MBit;Gesamt GByte;Gesamt GBit;KByte/s;KBit/s;MByte/s;MBit/s;GByte/s;GBit/s"
        )
        .unwrap();
    }

    writeln!(
        writer,
        "{};{};{:?};{};{};{};{};{};{};{};{};{};{};{};{}",
        timestamp,
        remote_addr,
        direction,
        block_size_kb,
        format_number(stats.duration, &locale),
        format_number(stats.total_mbytes, &locale),
        format_number(stats.total_mbits, &locale),
        format_number(stats.total_gbytes, &locale),
        format_number(stats.total_gbits, &locale),
        format_number(stats.kbytes_per_sec, &locale),
        format_number(stats.kbits_per_sec, &locale),
        format_number(stats.mbytes_per_sec, &locale),
        format_number(stats.mbits_per_sec, &locale),
        format_number(stats.gbytes_per_sec, &locale),
        format_number(stats.gbits_per_sec, &locale),
    )
    .unwrap();

    writer.flush().expect("Failed to flush CSV writer");
}

pub fn print_statistics_terminal(duration: f64, total_bytes: usize) {
    let stats = calculate_statistics(duration, total_bytes);
    write_statistics_terminal(&stats);
}

pub fn print_statistics(duration: f64, total_bytes: usize, direction: Direction, block_size_kb: usize, remote_addr: &str) {
    let stats = calculate_statistics(duration, total_bytes);
    write_statistics_terminal(&stats);
    write_statistics_csv(&stats, direction, block_size_kb, remote_addr);
}

pub fn format_number(value: f64, locale: &Locale) -> String {
    let whole = value.trunc() as u64;
    let fraction = value.fract();

    if fraction == 0.0 {
        return whole.to_formatted_string(locale);
    }

    let formatted_whole = whole.to_formatted_string(locale);
    let formatted_fraction = format!("{:.2}", fraction).trim_start_matches("0").replace(".", ",");

    format!("{}{}", formatted_whole, formatted_fraction)
}

pub fn format_duration_hms(start: Instant, end: Instant) -> String {
    let duration = end.duration_since(start);

    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn generate_test_sizes(max_size: usize) -> Vec<usize> {
    let mut sizes = Vec::new();
    let min_exponent = 12; // Start at 4 KiB
    let max_exponent = (max_size as f64).log2().floor() as u32;

    for exp in min_exponent..=max_exponent {
        sizes.push(1 << exp);
    }

    // Add max_size if it's not already a power of two in the list
    if *sizes.last().unwrap() != max_size {
        sizes.push(max_size);
    }

    sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    // 67.108.864

    #[test]
    fn test_generate_test_sizes() {
        let max_size = 100 * 1024 * 1024; // 100 MiB
        let sizes = generate_test_sizes(max_size);

        print!("sizes = {:#?}", sizes);

        assert!(!sizes.is_empty(), "Should generate at least one size");
        assert_eq!(*sizes.last().unwrap(), max_size, "Last size should be max_size");

        // All sizes except the last must be powers of two
        for &size in &sizes[..sizes.len() - 1] {
            assert!(size.is_power_of_two(), "Size {} is not power of two", size);
        }
    }
}
