use num_format::{Locale, ToFormattedString};
use tokio::time::Instant;

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

pub fn print_statistics(duration: f64, total_bytes: u64) {
    let locale = Locale::de;

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

    println!("• Dauer:");
    println!("   - {} min {} s", minutes, seconds);
    println!("   - {} Sekunden", format_number(duration, &locale));

    println!("• Übertragen");
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
