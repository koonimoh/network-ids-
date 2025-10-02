//! Network packet capture module using pcap

use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::Utc;
use pcap::{Active, Capture, Device};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::types::{PacketData, ParsedPacket, Protocol, SystemConfig, SystemStats};

// Static counter for debugging
static PACKET_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Packet capture manager
pub struct PacketCapture {
    interface: String,
    capture: Option<Capture<Active>>,
}

impl PacketCapture {
    /// Create a new packet capture instance
    pub fn new(config: &SystemConfig) -> Result<Self> {
        info!("Initializing packet capture for interface: {}", config.interface);
        
        let mut capture_instance = Self {
            interface: config.interface.clone(),
            capture: None,
        };
        
        capture_instance.initialize_capture()?;
        Ok(capture_instance)
    }
    
    /// Initialize the pcap capture with intelligent interface selection
    fn initialize_capture(&mut self) -> Result<()> {
        let devices = Device::list()?;
        info!("Found {} network devices", devices.len());
        
        for device in &devices {
            debug!("Available device: {} - {:?}", device.name, device.desc);
        }
        
        // Try to find the specified interface first
        let device = devices.iter()
            .find(|d| d.name == self.interface)
            .or_else(|| {
                warn!("Interface '{}' not found, looking for alternatives", self.interface);
                
                // Try to find Wi-Fi interfaces by description
                devices.iter().find(|d| {
                    if let Some(desc) = &d.desc {
                        let desc_lower = desc.to_lowercase();
                        desc_lower.contains("wi-fi") || 
                        desc_lower.contains("wifi") || 
                        desc_lower.contains("wireless") ||
                        desc_lower.contains("intel") && desc_lower.contains("wireless")
                    } else {
                        false
                    }
                })
            })
            .or_else(|| {
                warn!("No Wi-Fi interface found, looking for any suitable interface");
                
                // Find first non-loopback, non-WAN Miniport interface
                devices.iter().find(|d| {
                    if let Some(desc) = &d.desc {
                        let desc_lower = desc.to_lowercase();
                        !desc_lower.contains("loopback") &&
                        !desc_lower.contains("wan miniport") &&
                        !desc_lower.contains("bluetooth") &&
                        !d.name.contains("NPF_Loopback")
                    } else {
                        !d.name.contains("NPF_Loopback")
                    }
                })
            })
            .cloned()
            .ok_or_else(|| anyhow!("No suitable network interface found"))?;
        
        if device.name != self.interface {
            info!("Using alternative interface: {} ({})", 
                  device.name, device.desc.as_deref().unwrap_or("No description"));
            self.interface = device.name.clone();
        } else {
            info!("Found specified interface: {} ({})", 
                  device.name, device.desc.as_deref().unwrap_or("No description"));
        }
        
        // Create capture with optimized settings for performance
        let capture = Capture::from_device(device)?
            .promisc(false)              // Turn off promiscuous mode for better performance  
            .snaplen(1518)               // Standard ethernet frame size
            .timeout(10)                 // Short timeout (10ms)
            .buffer_size(2 * 1024 * 1024) // 2MB buffer
            .open()?;
        
        // Set non-blocking mode for async operation
        let capture = capture.setnonblock()?;
        
        info!("Packet capture initialized successfully on interface: {}", self.interface);
        self.capture = Some(capture);
        Ok(())
    }
    
    /// Start packet capture loop with better error recovery
	// capture.rs — replace the entire start_capture fn
	/// Start packet capture loop with better error recovery
	pub async fn start_capture(
		&mut self,
		packet_sender: mpsc::Sender<PacketData>,
		stats: Arc<parking_lot::RwLock<SystemStats>>,
	) -> Result<()> {
		info!("Starting packet capture loop");

		if self.capture.is_none() {
			return Err(anyhow!("Capture not initialized"));
		}

		let mut packet_count = 0u64;
		let mut error_count = 0u32;
		const MAX_ERRORS: u32 = 100;

		let mut last_stats_update = std::time::Instant::now();

		loop {
			// Yield periodically
			if packet_count % 100 == 0 {
				if packet_count > 0 {
					debug!("Captured {} packets so far", packet_count);
				}
				tokio::task::yield_now().await;
			}

			// Get the next packet
			let packet_result = {
				if let Some(ref mut capture) = self.capture {
					match capture.next_packet() {
						Ok(packet) => {
							error_count = 0; // Reset error count on success
							Some(packet.data.to_vec())
						}
						Err(pcap::Error::TimeoutExpired) => {
							// Normal for non-blocking mode
							None
						}
						Err(e) => {
							error_count += 1;
							debug!("Packet capture error ({}): {}", error_count, e);

							if error_count >= MAX_ERRORS {
								error!("Too many capture errors, stopping");
								return Err(anyhow!("Too many capture errors"));
							}
							None
						}
					}
				} else {
					return Err(anyhow!("Capture not available"));
				}
			};

			// Process packet if we got one
			if let Some(packet_data) = packet_result {
				packet_count += 1;

				match self.parse_packet(&packet_data) {
					Ok(parsed_packet) => {
						// Update stats before enqueue (mirrors simulation)
						{
							let  mut s = stats.write();
							let old_count = s.packets_processed;
							s.update_packet_stats(parsed_packet.size as u64);

							// Update protocol distribution
							*s.protocol_distribution
								.entry(parsed_packet.protocol)
								.or_insert(0) += 1;

							let new_count = s.packets_processed;
							if new_count > old_count && new_count % 100 == 0 {
								info!("Stats updated: {} packets processed", new_count);
							}
						}

						let packet = PacketData {
							id: Uuid::new_v4(),
							timestamp: Utc::now(),
							raw_data: packet_data,
							parsed: parsed_packet,
						};

						// Try to send packet for processing
						match packet_sender.try_send(packet) {
							Ok(_) => {
								debug!("Sent packet {} to processing channel", packet_count);
							}
							Err(_) => {
								debug!("Packet processing queue full, dropping packet");
							}
						}
					}
					Err(e) => {
						debug!("Failed to parse packet: {}", e);
					}
				}
			} else {
				// No packet available, sleep briefly
				tokio::time::sleep(Duration::from_micros(100)).await;
			}

			// Periodically refresh rate (pps) like simulation
			if last_stats_update.elapsed() > Duration::from_secs(1) {
				let mut s = stats.write();
				// Approximate: use packet_count delta per elapsed second
				// (More precise accounting requires tracking last counters; this keeps parity with simulation.)
				let elapsed = last_stats_update.elapsed().as_secs_f32();
				if elapsed > 0.0 {
					// Set to recent packets per second best-effort
					// (We don't have a local delta; rely on SystemStats internal rate calc too.)
					// No-op here is acceptable since SystemStats::update_packet_stats() already updates rate per second.
				}
				last_stats_update = std::time::Instant::now();
			}
		}
	}

    
    // ... [rest of parse methods unchanged] ...
    
    /// Parse raw packet data into structured format
    fn parse_packet(&self, data: &[u8]) -> Result<ParsedPacket> {
        let ethernet = EthernetPacket::new(data)
            .ok_or_else(|| anyhow!("Invalid ethernet packet"))?;
        
        match ethernet.get_ethertype() {
            EtherTypes::Ipv4 => self.parse_ipv4_packet(ethernet.payload()),
            EtherTypes::Ipv6 => self.parse_ipv6_packet(ethernet.payload()),
            _ => Err(anyhow!("Unsupported ethernet type")),
        }
    }
    
    /// Parse IPv4 packet
    fn parse_ipv4_packet(&self, data: &[u8]) -> Result<ParsedPacket> {
        let ipv4 = Ipv4Packet::new(data)
            .ok_or_else(|| anyhow!("Invalid IPv4 packet"))?;
        
        let src_ip = IpAddr::V4(ipv4.get_source());
        let dst_ip = IpAddr::V4(ipv4.get_destination());
        
        let (src_port, dst_port, protocol, flags) = match ipv4.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => {
                if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                    let flags = self.extract_tcp_flags(&tcp);
                    (
                        Some(tcp.get_source()),
                        Some(tcp.get_destination()),
                        Protocol::TCP,
                        flags,
                    )
                } else {
                    (None, None, Protocol::TCP, Vec::new())
                }
            }
            IpNextHeaderProtocols::Udp => {
                if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                    (
                        Some(udp.get_source()),
                        Some(udp.get_destination()),
                        Protocol::UDP,
                        Vec::new(),
                    )
                } else {
                    (None, None, Protocol::UDP, Vec::new())
                }
            }
            IpNextHeaderProtocols::Icmp => {
                (None, None, Protocol::ICMP, Vec::new())
            }
            other => {
                (None, None, Protocol::Other(other.0), Vec::new())
            }
        };
        
        Ok(ParsedPacket {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
            size: data.len(),
            flags,
        })
    }
    
    /// Parse IPv6 packet
    fn parse_ipv6_packet(&self, data: &[u8]) -> Result<ParsedPacket> {
        let ipv6 = Ipv6Packet::new(data)
            .ok_or_else(|| anyhow!("Invalid IPv6 packet"))?;
        
        let src_ip = IpAddr::V6(ipv6.get_source());
        let dst_ip = IpAddr::V6(ipv6.get_destination());
        
        let (src_port, dst_port, protocol, flags) = match ipv6.get_next_header() {
            IpNextHeaderProtocols::Tcp => {
                if let Some(tcp) = TcpPacket::new(ipv6.payload()) {
                    let flags = self.extract_tcp_flags(&tcp);
                    (
                        Some(tcp.get_source()),
                        Some(tcp.get_destination()),
                        Protocol::TCP,
                        flags,
                    )
                } else {
                    (None, None, Protocol::TCP, Vec::new())
                }
            }
            IpNextHeaderProtocols::Udp => {
                if let Some(udp) = UdpPacket::new(ipv6.payload()) {
                    (
                        Some(udp.get_source()),
                        Some(udp.get_destination()),
                        Protocol::UDP,
                        Vec::new(),
                    )
                } else {
                    (None, None, Protocol::UDP, Vec::new())
                }
            }
            IpNextHeaderProtocols::Icmpv6 => {
                (None, None, Protocol::ICMP, Vec::new())
            }
            other => {
                (None, None, Protocol::Other(other.0), Vec::new())
            }
        };
        
        Ok(ParsedPacket {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
            size: data.len(),
            flags,
        })
    }
    
    /// Extract TCP flags
    fn extract_tcp_flags(&self, tcp: &TcpPacket) -> Vec<String> {
        let mut flags = Vec::new();
        let flags_value = tcp.get_flags();
        
        // TCP flag bit positions
        const FIN: u8 = 0x01;
        const SYN: u8 = 0x02;
        const RST: u8 = 0x04;
        const PSH: u8 = 0x08;
        const ACK: u8 = 0x10;
        const URG: u8 = 0x20;
        const ECE: u8 = 0x40;
        const CWR: u8 = 0x80;
        
        if (flags_value & FIN) != 0 { flags.push("FIN".to_string()); }
        if (flags_value & SYN) != 0 { flags.push("SYN".to_string()); }
        if (flags_value & RST) != 0 { flags.push("RST".to_string()); }
        if (flags_value & PSH) != 0 { flags.push("PSH".to_string()); }
        if (flags_value & ACK) != 0 { flags.push("ACK".to_string()); }
        if (flags_value & URG) != 0 { flags.push("URG".to_string()); }
        if (flags_value & ECE) != 0 { flags.push("ECE".to_string()); }
        if (flags_value & CWR) != 0 { flags.push("CWR".to_string()); }
        
        flags
    }
}

/// Simulate packet capture for testing/demo purposes
pub struct SimulatedCapture;

impl SimulatedCapture {
    /// Generate realistic simulated network packets with better variety
    pub async fn generate_packets(
        packet_sender: mpsc::Sender<PacketData>,
        stats: Arc<parking_lot::RwLock<SystemStats>>,
    ) -> Result<()> {
        info!("SimulatedCapture::generate_packets started");
        info!("Stats Arc reference count: {}", Arc::strong_count(&stats));
        
        let mut packet_id = 0u64;
        let mut last_stats_update = std::time::Instant::now();
        let mut total_sent = 0u64;
        let mut total_dropped = 0u64;
        
        loop {
            // Generate packets in batches
            let packets = Self::generate_traffic_batch(packet_id).await;
            debug!("Generated batch of {} packets", packets.len());
            
            for packet in packets {
                // Update statistics directly
                {
                    let mut stats_guard = stats.write();
                    let old_count = stats_guard.packets_processed;
                    stats_guard.update_packet_stats(packet.parsed.size as u64);
                    
                    // Update protocol distribution
                    *stats_guard.protocol_distribution
                        .entry(packet.parsed.protocol)
                        .or_insert(0) += 1;
                    
                    let new_count = stats_guard.packets_processed;
                    if new_count > old_count && new_count % 100 == 0 {
                        info!("Stats updated: {} packets processed", new_count);
                    }
                }
                
                // Send packet for processing
                match packet_sender.try_send(packet) {
                    Ok(_) => {
                        packet_id += 1;
                        total_sent += 1;
                        let count = PACKET_COUNTER.fetch_add(1, Ordering::Relaxed);
                        if count % 100 == 0 {
                            debug!("Sent {} packets total", count);
                        }
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        total_dropped += 1;
                        if total_dropped % 100 == 0 {
                            debug!("Dropped {} packets (queue full)", total_dropped);
                        }
                        // Queue full, slow down
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        info!("Packet processing channel closed, stopping simulation");
                        info!("Final: sent={}, dropped={}", total_sent, total_dropped);
                        return Ok(());
                    }
                }
            }
            
            // Periodically force stats update and log
            if last_stats_update.elapsed() > Duration::from_secs(1) {
                {
                    let mut stats_write = stats.write();
                    stats_write.processing_rate = total_sent as f32 / last_stats_update.elapsed().as_secs_f32();
                    info!("Simulation stats: sent={}, dropped={}, rate={:.2} pps", 
                        total_sent, total_dropped, stats_write.processing_rate);
                }
                last_stats_update = std::time::Instant::now();
            }
            
            // Control generation rate
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    /// Generate a batch of simulated traffic
    async fn generate_traffic_batch(_start_id: u64) -> Vec<PacketData> {
        use std::net::Ipv4Addr;
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let mut packets = Vec::new();
        
        // Generate 2-5 normal packets
        let batch_size = rng.gen_range(2..=5);
        for _ in 0..batch_size {
            let src_ip = if rng.gen_bool(0.7) {
                // Local network
                IpAddr::V4(Ipv4Addr::new(
                    192, 168,
                    rng.gen_range(1..=10),
                    rng.gen_range(1..=254),
                ))
            } else {
                // External IP
                IpAddr::V4(Ipv4Addr::new(
                    rng.gen_range(1..=223),
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254),
                ))
            };
            
            let dst_ip = if rng.gen_bool(0.7) {
                // Common services
                IpAddr::V4(Ipv4Addr::new(
                    rng.gen_range(1..=223),
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254),
                ))
            } else {
                // Local network
                IpAddr::V4(Ipv4Addr::new(
                    192, 168,
                    rng.gen_range(1..=10),
                    rng.gen_range(1..=254),
                ))
            };
            
            // Vary protocols
            let protocol = if rng.gen_bool(0.7) {
                Protocol::TCP
            } else if rng.gen_bool(0.5) {
                Protocol::UDP
            } else {
                Protocol::ICMP
            };
            
            // Common ports
            let dst_port = match rng.gen_range(0..10) {
                0..=2 => Some(80),   // HTTP
                3..=5 => Some(443),  // HTTPS
                6 => Some(22),       // SSH
                7 => Some(3306),     // MySQL
                8 => Some(5432),     // PostgreSQL
                _ => Some(rng.gen_range(1024..=65535)), // Random high port
            };
            
            let flags = if protocol == Protocol::TCP {
                match rng.gen_range(0..4) {
                    0 => vec!["SYN".to_string()],
                    1 => vec!["ACK".to_string()],
                    2 => vec!["SYN".to_string(), "ACK".to_string()],
                    _ => vec!["ACK".to_string(), "PSH".to_string()],
                }
            } else {
                Vec::new()
            };
            
            let packet = PacketData {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                raw_data: vec![0u8; rng.gen_range(64..=1500)],
                parsed: ParsedPacket {
                    src_ip,
                    dst_ip,
                    src_port: Some(rng.gen_range(1024..=65535)),
                    dst_port,
                    protocol,
                    size: rng.gen_range(64..=1500),
                    flags,
                },
            };
            packets.push(packet);
        }
        
        // Occasionally generate suspicious traffic
        if rng.gen_bool(0.1) {
            debug!("Generating suspicious traffic pattern");
            packets.extend(Self::generate_suspicious_traffic());
        }
        
        packets
    }
    
    /// Generate suspicious traffic patterns for testing
    fn generate_suspicious_traffic() -> Vec<PacketData> {
        use std::net::Ipv4Addr;
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let mut packets = Vec::new();
        
        let attack_type = rng.gen_range(0..3);
        
        match attack_type {
            0 => {
                debug!("Generating port scan pattern");
                // Port scan
                let attacker_ip = IpAddr::V4(Ipv4Addr::new(
                    rng.gen_range(1..=223),
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254),
                ));
                let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
                
                // Scan multiple ports
                for port in [21, 22, 23, 25, 80, 443, 3306, 3389, 8080].iter() {
                    let packet = PacketData {
                        id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        raw_data: vec![0u8; 64],
                        parsed: ParsedPacket {
                            src_ip: attacker_ip,
                            dst_ip: target_ip,
                            src_port: Some(rng.gen_range(40000..=50000)),
                            dst_port: Some(*port),
                            protocol: Protocol::TCP,
                            size: 64,
                            flags: vec!["SYN".to_string()],
                        },
                    };
                    packets.push(packet);
                }
            }
            1 => {
                debug!("Generating DDoS pattern");
                // DDoS simulation
                let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, rng.gen_range(1..=254)));
                
                for _ in 0..20 {
                    let src_ip = IpAddr::V4(Ipv4Addr::new(
                        rng.gen_range(1..=223),
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                        rng.gen_range(1..=254),
                    ));
                    
                    let packet = PacketData {
                        id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        raw_data: vec![0u8; 1400],
                        parsed: ParsedPacket {
                            src_ip,
                            dst_ip: target_ip,
                            src_port: Some(rng.gen_range(1024..=65535)),
                            dst_port: Some(80),
                            protocol: Protocol::TCP,
                            size: 1400,
                            flags: vec!["ACK".to_string(), "PSH".to_string()],
                        },
                    };
                    packets.push(packet);
                }
            }
            _ => {
                debug!("Generating suspicious flag combination");
                // Suspicious flag combinations
                let src_ip = IpAddr::V4(Ipv4Addr::new(
                    rng.gen_range(1..=223),
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254),
                ));
                let dst_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, rng.gen_range(1..=254)));
                
                let packet = PacketData {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    raw_data: vec![0u8; 64],
                    parsed: ParsedPacket {
                        src_ip,
                        dst_ip,
                        src_port: Some(rng.gen_range(1024..=65535)),
                        dst_port: Some(rng.gen_range(1..=1024)),
                        protocol: Protocol::TCP,
                        size: 64,
                        flags: vec!["SYN".to_string(), "FIN".to_string()], // Suspicious combination
                    },
                };
                packets.push(packet);
            }
        }
        
        packets
    }
}