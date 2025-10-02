//! Core data types and structures for the Network IDS

use std::collections::HashMap;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Network interface to monitor
    pub interface: String,
    /// Detection sensitivity (0.0-1.0)
    pub sensitivity: f32,
    /// Maximum packets per second to process
    pub max_pps: u64,
    /// ML model configuration
    pub ml_config: MLConfig,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Use simulation mode (for testing/demo)
    pub use_simulation: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            interface: "Wi-Fi".to_string(),
            sensitivity: 0.7,
            max_pps: 10000,
            ml_config: MLConfig::default(),
            alert_thresholds: AlertThresholds::default(),
            use_simulation: false, // Will be auto-detected on Windows
        }
    }
}

/// Machine learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    /// Model update frequency (seconds)
    pub update_frequency: u64,
    /// Batch size for training
    pub batch_size: usize,
    /// Learning rate
    pub learning_rate: f32,
    /// Feature window size
    pub window_size: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            update_frequency: 300, // 5 minutes
            batch_size: 128,
            learning_rate: 0.001,
            window_size: 100,
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Anomaly score threshold for alerts
    pub anomaly_threshold: f32,
    /// Minimum confidence for alerts
    pub min_confidence: f32,
    /// Rate limiting (alerts per minute)
    pub max_alerts_per_minute: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            anomaly_threshold: 0.8,
            min_confidence: 0.7,
            max_alerts_per_minute: 10,
        }
    }
}

/// Raw packet data
#[derive(Debug, Clone)]
pub struct PacketData {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub raw_data: Vec<u8>,
    pub parsed: ParsedPacket,
}

/// Parsed packet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPacket {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: Protocol,
    pub size: usize,
    pub flags: Vec<String>,
}

/// Network protocol types with Hash trait
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::TCP => write!(f, "TCP"),
            Protocol::UDP => write!(f, "UDP"),
            Protocol::ICMP => write!(f, "ICMP"),
            Protocol::Other(n) => write!(f, "Protocol({})", n),
        }
    }
}

/// Network flow features for ML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowFeatures {
    pub flow_id: String,
    pub duration: f32,
    pub packet_count: u32,
    pub byte_count: u64,
    pub packets_per_second: f32,
    pub bytes_per_second: f32,
    pub avg_packet_size: f32,
    pub protocol_distribution: HashMap<Protocol, u32>,
    pub port_entropy: f32,
    pub inter_arrival_times: Vec<f32>,
    pub packet_size_variance: f32,
    pub flag_patterns: Vec<String>,
}

/// Threat alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAlert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,
    pub threat_type: ThreatType,
    pub confidence: f32,
    pub anomaly_score: f32,
    pub source_ip: IpAddr,
    pub target_ip: Option<IpAddr>,
    pub affected_ports: Vec<u16>,
    pub description: String,
    pub explanation: ThreatExplanation,
    pub raw_packets: Vec<Uuid>,
}

/// Threat severity levels with Hash trait
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

/// Types of threats detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    PortScan,
    DDoS,
    Anomalous,
    Suspicious,
    MalformedPacket,
    UnusualTraffic,
    PotentialIntrusion,
}

impl std::fmt::Display for ThreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatType::PortScan => write!(f, "Port Scan"),
            ThreatType::DDoS => write!(f, "DDoS Attack"),
            ThreatType::Anomalous => write!(f, "Anomalous Behavior"),
            ThreatType::Suspicious => write!(f, "Suspicious Activity"),
            ThreatType::MalformedPacket => write!(f, "Malformed Packet"),
            ThreatType::UnusualTraffic => write!(f, "Unusual Traffic Pattern"),
            ThreatType::PotentialIntrusion => write!(f, "Potential Intrusion"),
        }
    }
}

/// Explanation of why a threat was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatExplanation {
    pub primary_indicators: Vec<String>,
    pub feature_importance: HashMap<String, f32>,
    pub similar_incidents: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// System statistics with thread-safe updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub start_time: DateTime<Utc>,
    pub packets_processed: u64,
    pub bytes_processed: u64,
    pub threats_detected: u64,
    pub processing_rate: f32,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub active_flows: u32,
    pub alert_counts: HashMap<Severity, u32>,
    pub protocol_distribution: HashMap<Protocol, u64>,
    pub top_talkers: Vec<(IpAddr, u64)>,
    #[serde(skip, default = "std::time::Instant::now")]
    last_rate_calculation: std::time::Instant,
    #[serde(skip, default)]
    last_packet_count: u64,
}

impl SystemStats {
    pub fn new() -> Self {
        Self {
            start_time: Utc::now(),
            packets_processed: 0,
            bytes_processed: 0,
            threats_detected: 0,
            processing_rate: 0.0,
            memory_usage: 0,
            cpu_usage: 0.0,
            active_flows: 0,
            alert_counts: HashMap::new(),
            protocol_distribution: HashMap::new(),
            top_talkers: Vec::new(),
            last_rate_calculation: std::time::Instant::now(),
            last_packet_count: 0,
        }
    }
    
    pub fn update_packet_stats(&mut self, packet_size: u64) {
        self.packets_processed += 1;
        self.bytes_processed += packet_size;
        
        // Update processing rate every second
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_rate_calculation).as_secs_f32();
        if elapsed >= 1.0 {
            let packets_delta = self.packets_processed - self.last_packet_count;
            self.processing_rate = packets_delta as f32 / elapsed;
            self.last_rate_calculation = now;
            self.last_packet_count = self.packets_processed;
        }
    }
    
    pub fn increment_threat_count(&mut self, severity: Severity) {
        self.threats_detected += 1;
        *self.alert_counts.entry(severity).or_insert(0) += 1;
    }
}

impl Default for SystemStats {
    fn default() -> Self {
        Self::new()
    }
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            timestamp: Utc::now(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// AI query request from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIQueryRequest {
    pub query: String,
    pub provider: String,
    pub conversation_history: Vec<ChatMessage>,
}

/// AI query response to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIQueryResponse {
    pub response: String,
    pub model_used: String,
    pub tokens_used: Option<u32>,
}