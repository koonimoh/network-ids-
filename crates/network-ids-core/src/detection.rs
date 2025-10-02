//! Threat detection and analysis engine

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::Utc;
use dashmap::DashMap;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::ml::MLEngine;
use crate::types::{
    FlowFeatures, PacketData, Protocol, Severity, SystemStats, ThreatAlert, ThreatExplanation,
    ThreatType,
};




/// Network flow tracking
#[derive(Debug, Clone)]
pub struct NetworkFlow {
    flow_id: String,
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: Option<u16>,
    dst_port: Option<u16>,
    protocol: Protocol,
    packets: Vec<PacketData>,
    start_time: Instant,
    last_seen: Instant,
    byte_count: u64,
    flags_seen: Vec<String>,
}

impl NetworkFlow {
    fn new(packet: &PacketData) -> Self {
        let flow_id = format!(
            "{}:{:?}-{}:{:?}-{}",
            packet.parsed.src_ip,
            packet.parsed.src_port,
            packet.parsed.dst_ip,
            packet.parsed.dst_port,
            packet.parsed.protocol
        );

        Self {
            flow_id,
            src_ip: packet.parsed.src_ip,
            dst_ip: packet.parsed.dst_ip,
            src_port: packet.parsed.src_port,
            dst_port: packet.parsed.dst_port,
            protocol: packet.parsed.protocol,
            packets: vec![packet.clone()],
            start_time: Instant::now(),
            last_seen: Instant::now(),
            byte_count: packet.parsed.size as u64,
            flags_seen: packet.parsed.flags.clone(),
        }
    }

    fn add_packet(&mut self, packet: &PacketData) {
        self.packets.push(packet.clone());
        self.last_seen = Instant::now();
        self.byte_count += packet.parsed.size as u64;
        
        // Merge unique flags
        for flag in &packet.parsed.flags {
            if !self.flags_seen.contains(flag) {
                self.flags_seen.push(flag.clone());
            }
        }
    }

    fn packet_count(&self) -> u32 {
        self.packets.len() as u32
    }
    
    #[allow(dead_code)]
    fn get_src_port(&self) -> Option<u16> {
        self.src_port
    }
    
    #[allow(dead_code)]
    fn get_protocol(&self) -> &Protocol {
        &self.protocol
    }

    fn to_features(&self) -> FlowFeatures {
        let duration = self.last_seen.duration_since(self.start_time).as_secs_f32();
        let packet_count = self.packets.len() as u32;
        let packets_per_second = if duration > 0.0 {
            packet_count as f32 / duration
        } else {
            0.0
        };
        let bytes_per_second = if duration > 0.0 {
            self.byte_count as f32 / duration
        } else {
            0.0
        };
        let avg_packet_size = if packet_count > 0 {
            self.byte_count as f32 / packet_count as f32
        } else {
            0.0
        };

        // Calculate inter-arrival times
        let mut inter_arrival_times = Vec::new();
        for i in 1..self.packets.len() {
            let diff = self.packets[i]
                .timestamp
                .signed_duration_since(self.packets[i - 1].timestamp)
                .num_milliseconds() as f32
                / 1000.0;
            inter_arrival_times.push(diff);
        }

        // Calculate packet size variance
        let sizes: Vec<f32> = self.packets.iter().map(|p| p.parsed.size as f32).collect();
        let mean_size = avg_packet_size;
        let packet_size_variance = if sizes.len() > 1 {
            sizes
                .iter()
                .map(|&size| (size - mean_size).powi(2))
                .sum::<f32>()
                / (sizes.len() - 1) as f32
        } else {
            0.0
        };

        // Protocol distribution
        let mut protocol_distribution = HashMap::new();
        for packet in &self.packets {
            *protocol_distribution
                .entry(packet.parsed.protocol)
                .or_insert(0) += 1;
        }

        // Port entropy calculation
        let mut port_counts = HashMap::new();
        for packet in &self.packets {
            if let Some(port) = packet.parsed.dst_port {
                *port_counts.entry(port).or_insert(0) += 1;
            }
        }

        let port_entropy = if port_counts.len() > 1 {
            let total = port_counts.values().sum::<u32>() as f32;
            port_counts
                .values()
                .map(|&count| {
                    let p = count as f32 / total;
                    -p * p.log2()
                })
                .sum()
        } else {
            0.0
        };

        FlowFeatures {
            flow_id: self.flow_id.clone(),
            duration,
            packet_count,
            byte_count: self.byte_count,
            packets_per_second,
            bytes_per_second,
            avg_packet_size,
            protocol_distribution,
            port_entropy,
            inter_arrival_times,
            packet_size_variance,
            flag_patterns: self.flags_seen.clone(),
        }
    }
}

/// Threat detection patterns
pub struct ThreatPatterns;

impl ThreatPatterns {
    /// Detect port scanning behavior
    pub fn detect_port_scan(flows: &[&NetworkFlow]) -> Option<ThreatAlert> {
        // Group flows by source IP
        let mut ip_port_map: HashMap<IpAddr, Vec<u16>> = HashMap::new();
        
        for flow in flows {
            if let Some(dst_port) = flow.dst_port {
                ip_port_map
                    .entry(flow.src_ip)
                    .or_default()
                    .push(dst_port);
            }
        }

        // Look for IPs scanning multiple ports
        for (src_ip, ports) in ip_port_map {
            let unique_ports: std::collections::HashSet<_> = ports.iter().collect();
            
            if unique_ports.len() >= 5 {
                // Potential port scan detected
                let confidence = (unique_ports.len() as f32 / 100.0).min(1.0);
                
                // Clone the ports vector to avoid move issues
                let ports_clone = ports.clone();
                
                return Some(ThreatAlert {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    severity: if unique_ports.len() > 20 {
                        Severity::High
                    } else if unique_ports.len() > 10 {
                        Severity::Medium
                    } else {
                        Severity::Low
                    },
                    threat_type: ThreatType::PortScan,
                    confidence,
                    anomaly_score: confidence,
                    source_ip: src_ip,
                    target_ip: flows.first().map(|f| f.dst_ip),
                    affected_ports: ports_clone,
                    description: format!(
                        "Port scan detected from {} targeting {} unique ports",
                        src_ip,
                        unique_ports.len()
                    ),
                    explanation: ThreatExplanation {
                        primary_indicators: vec![
                            format!("Multiple port connections: {}", unique_ports.len()),
                            "Sequential port access pattern".to_string(),
                        ],
                        feature_importance: [
                            ("unique_ports".to_string(), confidence),
                            ("connection_pattern".to_string(), 0.8),
                        ]
                        .into_iter()
                        .collect(),
                        similar_incidents: vec![
                            "Known port scanning signature".to_string(),
                        ],
                        recommended_actions: vec![
                            "Block source IP address".to_string(),
                            "Monitor for further scanning activity".to_string(),
                            "Check target systems for vulnerabilities".to_string(),
                        ],
                    },
                    raw_packets: flows.iter()
                        .filter(|f| f.src_ip == src_ip)
                        .flat_map(|f| f.packets.iter().map(|p| p.id))
                        .collect(),
                });
            }
        }

        None
    }

    /// Detect DDoS patterns
    pub fn detect_ddos(flows: &[&NetworkFlow]) -> Option<ThreatAlert> {
        // Group by target IP
        let mut target_traffic: HashMap<IpAddr, (u32, u64)> = HashMap::new();
        
        for flow in flows {
            let entry = target_traffic.entry(flow.dst_ip).or_insert((0, 0));
            entry.0 += flow.packet_count();
            entry.1 += flow.byte_count;
        }

        // Look for unusually high traffic to single targets
        for (target_ip, (packet_count, byte_count)) in target_traffic {
            if packet_count > 1000 || byte_count > 10_000_000 {
                // Potential DDoS
                let confidence = ((packet_count as f32 / 10000.0).min(1.0) +
                                 (byte_count as f32 / 100_000_000.0).min(1.0)) / 2.0;
                
                return Some(ThreatAlert {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    severity: if packet_count > 5000 || byte_count > 50_000_000 {
                        Severity::Critical
                    } else if packet_count > 2000 || byte_count > 20_000_000 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    threat_type: ThreatType::DDoS,
                    confidence,
                    anomaly_score: confidence,
                    source_ip: flows.first().map(|f| f.src_ip).unwrap_or(target_ip),
                    target_ip: Some(target_ip),
                    affected_ports: flows.iter()
                        .filter(|f| f.dst_ip == target_ip)
                        .filter_map(|f| f.dst_port)
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect(),
                    description: format!(
                        "Potential DDoS attack detected against {} - {} packets, {} bytes",
                        target_ip, packet_count, byte_count
                    ),
                    explanation: ThreatExplanation {
                        primary_indicators: vec![
                            format!("High packet volume: {} packets", packet_count),
                            format!("High bandwidth usage: {} bytes", byte_count),
                            "Multiple source IPs targeting single destination".to_string(),
                        ],
                        feature_importance: [
                            ("packet_volume".to_string(), 0.9),
                            ("bandwidth_usage".to_string(), 0.8),
                            ("source_diversity".to_string(), 0.7),
                        ]
                        .into_iter()
                        .collect(),
                        similar_incidents: vec![
                            "Volume-based DDoS pattern".to_string(),
                        ],
                        recommended_actions: vec![
                            "Activate DDoS protection measures".to_string(),
                            "Rate limit incoming connections".to_string(),
                            "Contact ISP for upstream filtering".to_string(),
                            "Monitor target system performance".to_string(),
                        ],
                    },
                    raw_packets: flows.iter()
                        .filter(|f| f.dst_ip == target_ip)
                        .flat_map(|f| f.packets.iter().map(|p| p.id))
                        .collect(),
                });
            }
        }

        None
    }

    /// Detect suspicious flag combinations
    pub fn detect_suspicious_flags(flow: &NetworkFlow) -> Option<ThreatAlert> {
        let flags_str = flow.flags_seen.join(",");
        
        // Check for suspicious flag combinations
        let is_suspicious = flow.flags_seen.contains(&"SYN".to_string()) && 
                           flow.flags_seen.contains(&"FIN".to_string()) ||
                           flow.flags_seen.iter().filter(|&flag| flag == "SYN").count() > 10;

        if is_suspicious {
            let confidence = 0.6;
            
            return Some(ThreatAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: Severity::Medium,
                threat_type: ThreatType::Suspicious,
                confidence,
                anomaly_score: confidence,
                source_ip: flow.src_ip,
                target_ip: Some(flow.dst_ip),
                affected_ports: flow.dst_port.into_iter().collect(),
                description: format!(
                    "Suspicious TCP flag combination detected: {}",
                    flags_str
                ),
                explanation: ThreatExplanation {
                    primary_indicators: vec![
                        format!("Unusual flag combination: {}", flags_str),
                        "Potential TCP stack fingerprinting".to_string(),
                    ],
                    feature_importance: [
                        ("flag_pattern".to_string(), 0.8),
                        ("connection_behavior".to_string(), 0.6),
                    ]
                    .into_iter()
                    .collect(),
                    similar_incidents: vec![
                        "TCP flag manipulation attempt".to_string(),
                    ],
                    recommended_actions: vec![
                        "Monitor source IP for additional suspicious activity".to_string(),
                        "Check firewall rules for flag filtering".to_string(),
                    ],
                },
                raw_packets: flow.packets.iter().map(|p| p.id).collect(),
            });
        }

        None
    }
}

/// Main threat detection engine
pub struct DetectionEngine {
    ml_engine: Arc<MLEngine>,
    alert_sender: broadcast::Sender<ThreatAlert>,
    active_flows: Arc<DashMap<String, NetworkFlow>>,
    recent_alerts: Arc<parking_lot::RwLock<VecDeque<ThreatAlert>>>,
    flow_timeout: Duration,
}

impl DetectionEngine {
    /// Create a new detection engine
    pub fn new(
        ml_engine: Arc<MLEngine>,
        alert_sender: broadcast::Sender<ThreatAlert>,
    ) -> Result<Self> {
        Ok(Self {
            ml_engine,
            alert_sender,
            active_flows: Arc::new(DashMap::new()),
            recent_alerts: Arc::new(parking_lot::RwLock::new(VecDeque::new())),
            flow_timeout: Duration::from_secs(300), // 5 minutes
        })
    }

    /// Process incoming packets for threat detection
    pub async fn process_packets(
        &self,
        mut packet_receiver: mpsc::Receiver<PacketData>,
        stats: Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        info!("Starting threat detection engine");

        // Start flow cleanup task
        let flows_for_cleanup = Arc::clone(&self.active_flows);
        let cleanup_timeout = self.flow_timeout;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_expired_flows(&flows_for_cleanup, cleanup_timeout);
            }
        });

        while let Some(packet) = packet_receiver.recv().await {
            if let Err(e) = self.process_single_packet(packet, &stats).await {
                warn!("Error processing packet: {}", e);
            }
        }

        Ok(())
    }


	/// Process a single packet
    async fn process_single_packet(
        &self,
        packet: PacketData,
        stats: &Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        // Update packet statistics properly
        {
            let mut stats_guard = stats.write();
            stats_guard.update_packet_stats(packet.parsed.size as u64);
            
            // Update protocol distribution
            *stats_guard.protocol_distribution.entry(packet.parsed.protocol).or_insert(0) += 1;
            
            // Update active flows count (assuming it's u32 based on the error)
            stats_guard.active_flows = self.active_flows.len() as u32;
            
            // Update top talkers - handle the Vec<(IpAddr, u64)> type correctly
            let src_ip = packet.parsed.src_ip;
            let dst_ip = packet.parsed.dst_ip;
            let packet_size = packet.parsed.size as u64;
            
            // Find and update existing entries or add new ones
            let mut src_found = false;
            let mut dst_found = false;
            
            for (ip, bytes) in &mut stats_guard.top_talkers {
                if *ip == src_ip {
                    *bytes += packet_size;
                    src_found = true;
                } else if *ip == dst_ip {
                    *bytes += packet_size;
                    dst_found = true;
                }
            }
            
            if !src_found {
                stats_guard.top_talkers.push((src_ip, packet_size));
            }
            if !dst_found && dst_ip != src_ip {
                stats_guard.top_talkers.push((dst_ip, packet_size));
            }
            
            // Keep only top 10 talkers for performance
            if stats_guard.top_talkers.len() > 20 {
                stats_guard.top_talkers.sort_by(|a, b| b.1.cmp(&a.1));
                stats_guard.top_talkers.truncate(10);
            }
        }
        
        // Generate flow ID
        let flow_id = format!(
            "{}:{:?}-{}:{:?}-{}",
            packet.parsed.src_ip,
            packet.parsed.src_port,
            packet.parsed.dst_ip,
            packet.parsed.dst_port,
            packet.parsed.protocol
        );

        // Update or create flow
        let mut flow_updated = false;
        self.active_flows
            .entry(flow_id.clone())
            .and_modify(|flow| {
                flow.add_packet(&packet);
                flow_updated = true;
            })
            .or_insert_with(|| NetworkFlow::new(&packet));

        // Update active flows count again after potential new flow creation
        {
            let mut stats_guard = stats.write();
            stats_guard.active_flows = self.active_flows.len() as u32;
        }

        // Periodically analyze flows for threats
        if flow_updated {
            if let Some(flow) = self.active_flows.get(&flow_id) {
                // Extract features and run ML detection
                if flow.packets.len() >= 5 {
                    let features = flow.to_features();
                    
                    match self.ml_engine.predict(&features) {
                        Ok(anomaly_score) => {
                            if anomaly_score > 0.7 {
                                // High anomaly score - create alert
                                self.create_ml_alert(&*flow, anomaly_score, stats).await?;
                            }
                        }
                        Err(e) => {
                            debug!("ML prediction failed: {}", e);
                        }
                    }
                }
                
                // Run rule-based detection
                self.run_rule_based_detection(&*flow, stats).await?;
            }
        }

        // Periodically run global analysis
        if self.active_flows.len() % 100 == 0 {
            self.run_global_analysis(stats).await?;
        }

        Ok(())
    }

    /// Create ML-based threat alert
    async fn create_ml_alert(
        &self,
        flow: &NetworkFlow,
        anomaly_score: f32,
        stats: &Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        let severity = if anomaly_score > 0.9 {
            Severity::High
        } else if anomaly_score > 0.8 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let alert = ThreatAlert {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity,
            threat_type: ThreatType::Anomalous,
            confidence: anomaly_score,
            anomaly_score,
            source_ip: flow.src_ip,
            target_ip: Some(flow.dst_ip),
            affected_ports: flow.dst_port.into_iter().collect(),
            description: format!(
                "ML-detected anomalous network behavior (score: {:.3})",
                anomaly_score
            ),
            explanation: ThreatExplanation {
                primary_indicators: vec![
                    format!("High anomaly score: {:.3}", anomaly_score),
                    "Unusual traffic pattern detected by neural network".to_string(),
                ],
                feature_importance: [
                    ("ml_anomaly_score".to_string(), anomaly_score),
                    ("traffic_pattern".to_string(), 0.8),
                ]
                .into_iter()
                .collect(),
                similar_incidents: vec![
                    "Previously unseen traffic pattern".to_string(),
                ],
                recommended_actions: vec![
                    "Investigate source IP activity".to_string(),
                    "Monitor for pattern evolution".to_string(),
                    "Consider adding to watchlist".to_string(),
                ],
            },
            raw_packets: flow.packets.iter().map(|p| p.id).collect(),
        };

        self.send_alert(alert, stats).await
    }

    /// Run rule-based detection on a flow
    async fn run_rule_based_detection(
        &self,
        flow: &NetworkFlow,
        stats: &Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        // Check for suspicious flag patterns
        if let Some(alert) = ThreatPatterns::detect_suspicious_flags(flow) {
            self.send_alert(alert, stats).await?;
        }

        Ok(())
    }

    /// Run global analysis across all flows
    async fn run_global_analysis(
        &self,
        stats: &Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        let flows: Vec<_> = self.active_flows.iter().map(|entry| entry.value().clone()).collect();
        let flow_refs: Vec<_> = flows.iter().collect();

        // Check for port scans
        if let Some(alert) = ThreatPatterns::detect_port_scan(&flow_refs) {
            self.send_alert(alert, stats).await?;
        }

        // Check for DDoS
        if let Some(alert) = ThreatPatterns::detect_ddos(&flow_refs) {
            self.send_alert(alert, stats).await?;
        }

        Ok(())
    }

    /// Send threat alert
    async fn send_alert(
        &self,
        alert: ThreatAlert,
        stats: &Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        // Update statistics
        {
            let mut stats_guard = stats.write();
            stats_guard.increment_threat_count(alert.severity);
        }

        // Add to recent alerts
        {
            let mut recent = self.recent_alerts.write();
            recent.push_back(alert.clone());
            
            // Keep only last 1000 alerts
            if recent.len() > 100 {
                recent.pop_front();
            }
        }

        // Send alert
        if let Err(e) = self.alert_sender.send(alert.clone()) {
            warn!("Failed to send alert: {}", e);
        } else {
            info!(
                "Threat detected: {} from {} (severity: {}, confidence: {:.2})",
                alert.threat_type,
                alert.source_ip,
                alert.severity,
                alert.confidence
            );
        }

        Ok(())
    }

    /// Clean up expired flows
    fn cleanup_expired_flows(
        flows: &DashMap<String, NetworkFlow>,
        timeout: Duration,
    ) {
        let now = Instant::now();
        let expired_keys: Vec<_> = flows
            .iter()
            .filter_map(|entry| {
                if now.duration_since(entry.value().last_seen) > timeout {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        let expired_count = expired_keys.len();
        
        for key in expired_keys {
            flows.remove(&key);
        }

        debug!("Cleaned up {} expired flows, {} active", expired_count, flows.len());
    }

    /// Get recent alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<ThreatAlert> {
        let recent = self.recent_alerts.read();
        recent
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get active flow count
    pub fn get_active_flow_count(&self) -> usize {
        self.active_flows.len()
    }
	
	
	/// Get active flows for display
pub fn get_active_flows(&self) -> Vec<serde_json::Value> {
    use serde_json::json;
    
    self.active_flows
        .iter()
        .take(50) // Limit to 50 for performance
        .map(|entry| {
            let flow = entry.value();
            let duration = flow.last_seen.duration_since(flow.start_time).as_secs();
            
            json!({
                "flow_id": flow.flow_id,
                "src_ip": flow.src_ip.to_string(),
                "dst_ip": flow.dst_ip.to_string(),
                "src_port": flow.src_port,
                "dst_port": flow.dst_port,
                "protocol": format!("{:?}", flow.protocol),
                "packets": flow.packet_count(),
                "bytes": flow.byte_count,
                "duration": duration,
                "flags": flow.flags_seen.clone()
            })
        })
        .collect()
}
	
	
}