//! Utility functions and helpers

use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Format bytes into human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if size >= 100.0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else if size >= 10.0 {
        format!("{:.1} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration in a human readable way
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        if secs == 0 {
            format!("{}m", minutes)
        } else {
            format!("{}m {}s", minutes, secs)
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        if minutes == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, minutes)
        }
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        if hours == 0 {
            format!("{}d", days)
        } else {
            format!("{}d {}h", days, hours)
        }
    }
}

/// Validate IP address string
pub fn validate_ip_address(ip: &str) -> Result<std::net::IpAddr> {
    ip.parse::<std::net::IpAddr>()
        .map_err(|e| anyhow::anyhow!("Invalid IP address '{}': {}", ip, e))
}

/// Validate port number
pub fn validate_port(port: u32) -> Result<u16> {
    if port > 65535 {
        return Err(anyhow::anyhow!("Port number {} exceeds maximum (65535)", port));
    }
    Ok(port as u16)
}

/// Calculate percentage change
pub fn percentage_change(old_value: f64, new_value: f64) -> f64 {
    if old_value == 0.0 {
        if new_value == 0.0 {
            0.0
        } else {
            100.0
        }
    } else {
        ((new_value - old_value) / old_value) * 100.0
    }
}

/// Clamp value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Generate a random string of specified length
pub fn random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90061), "1d 1h");
    }

    #[test]
    fn test_percentage_change() {
        assert_eq!(percentage_change(100.0, 110.0), 10.0);
        assert_eq!(percentage_change(100.0, 90.0), -10.0);
        assert_eq!(percentage_change(0.0, 50.0), 100.0);
        assert_eq!(percentage_change(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 1, 10), 5);
        assert_eq!(clamp(0, 1, 10), 1);
        assert_eq!(clamp(15, 1, 10), 10);
    }
}