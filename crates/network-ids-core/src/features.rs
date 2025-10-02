//! Feature extraction and engineering module

use std::collections::HashMap;
use anyhow::Result;
use crate::types::{PacketData, FlowFeatures};

/// Extract features from network packets for ML analysis
pub struct FeatureExtractor;

impl FeatureExtractor {
    /// Create flow features from a sequence of packets
    pub fn extract_flow_features(packets: &[PacketData]) -> Result<FlowFeatures> {
        if packets.is_empty() {
            return Err(anyhow::anyhow!("Cannot extract features from empty packet sequence"));
        }

        let first_packet = &packets[0];
        let last_packet = packets.last().unwrap();
        
        // Basic flow identification
        let flow_id = format!(
            "{}:{:?}-{}:{:?}-{}",
            first_packet.parsed.src_ip,
            first_packet.parsed.src_port,
            first_packet.parsed.dst_ip,
            first_packet.parsed.dst_port,
            first_packet.parsed.protocol
        );

        // Time-based features
        let duration = last_packet.timestamp
            .signed_duration_since(first_packet.timestamp)
            .num_milliseconds() as f32 / 1000.0;
        
        let packet_count = packets.len() as u32;
        let byte_count: u64 = packets.iter().map(|p| p.parsed.size as u64).sum();
        
        let packets_per_second = if duration > 0.0 {
            packet_count as f32 / duration
        } else {
            0.0
        };
        
        let bytes_per_second = if duration > 0.0 {
            byte_count as f32 / duration
        } else {
            0.0
        };
        
        let avg_packet_size = if packet_count > 0 {
            byte_count as f32 / packet_count as f32
        } else {
            0.0
        };

        // Protocol distribution
        let mut protocol_distribution = HashMap::new();
        for packet in packets {
            *protocol_distribution.entry(packet.parsed.protocol).or_insert(0) += 1;
        }

        // Port entropy calculation
        let mut port_counts = HashMap::new();
        for packet in packets {
            if let Some(port) = packet.parsed.dst_port {
                *port_counts.entry(port).or_insert(0) += 1;
            }
        }

        let port_entropy = calculate_entropy(&port_counts);

        // Inter-arrival times
        let mut inter_arrival_times = Vec::new();
        for i in 1..packets.len() {
            let diff = packets[i].timestamp
                .signed_duration_since(packets[i-1].timestamp)
                .num_milliseconds() as f32 / 1000.0;
            inter_arrival_times.push(diff.max(0.0));
        }

        // Packet size variance
        let sizes: Vec<f32> = packets.iter().map(|p| p.parsed.size as f32).collect();
        let packet_size_variance = calculate_variance(&sizes);

        // Flag patterns
        let flag_patterns: Vec<String> = packets.iter()
            .flat_map(|p| p.parsed.flags.iter().cloned())
            .collect();

        Ok(FlowFeatures {
            flow_id,
            duration,
            packet_count,
            byte_count,
            packets_per_second,
            bytes_per_second,
            avg_packet_size,
            protocol_distribution,
            port_entropy,
            inter_arrival_times,
            packet_size_variance,
            flag_patterns,
        })
    }
}

/// Calculate Shannon entropy
fn calculate_entropy(counts: &HashMap<u16, u32>) -> f32 {
    if counts.len() <= 1 {
        return 0.0;
    }

    let total: u32 = counts.values().sum();
    if total == 0 {
        return 0.0;
    }

    counts.values()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = count as f32 / total as f32;
            -p * p.log2()
        })
        .sum()
}

/// Calculate variance of a series
fn calculate_variance(values: &[f32]) -> f32 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values.iter()
        .map(|&value| (value - mean).powi(2))
        .sum::<f32>() / (values.len() - 1) as f32;
    
    variance
}