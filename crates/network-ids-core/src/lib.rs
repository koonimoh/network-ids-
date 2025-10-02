//! # Network IDS Core
//! 
//! Core machine learning and network intrusion detection system.
//! Built with Rust 2024 edition for maximum performance and safety.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod capture;
pub mod detection;
pub mod features;
pub mod ml;
pub mod types;
pub mod utils;

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error, debug};

use crate::types::{PacketData, ThreatAlert, SystemConfig, SystemStats};

use sysinfo::System; // 0.30+: methods are inherent, no *Ext traits





/// Main network intrusion detection system
pub struct NetworkIDS {
    config: Arc<SystemConfig>,
    stats: Arc<parking_lot::RwLock<SystemStats>>,
    alert_sender: broadcast::Sender<ThreatAlert>,
    _alert_receiver: broadcast::Receiver<ThreatAlert>, // Keep one receiver alive
    shutdown_token: CancellationToken,
    detection_engine: Option<Arc<detection::DetectionEngine>>,
}

impl NetworkIDS {
    /// Create a new Network IDS instance
    pub fn new(config: SystemConfig) -> Result<Self> {
        info!("Creating new NetworkIDS instance");
        debug!("Config: {:?}", config);
        
        let (alert_sender, alert_receiver) = broadcast::channel(1000);
        let stats = SystemStats::new();
        
        info!("NetworkIDS instance created successfully");
        
        Ok(Self {
            config: Arc::new(config),
            stats: Arc::new(parking_lot::RwLock::new(stats)),
            alert_sender,
            _alert_receiver: alert_receiver,
            shutdown_token: CancellationToken::new(),
            detection_engine: None,
        })
    }
    

	pub async fn start(&mut self) -> Result<()> {
		info!("Starting Network IDS system");
		debug!("Current stats before start: {:?}", self.stats.read());

		// Initialize ML models
		info!("Initializing ML engine...");
		let ml_engine = ml::MLEngine::new(&self.config).await?;
		let ml_engine = Arc::new(ml_engine);
		info!("ML engine initialized successfully");

		// Initialize threat detection engine
		info!("Initializing detection engine...");
		let detection_engine = detection::DetectionEngine::new(
			Arc::clone(&ml_engine),
			self.alert_sender.clone(),
		)?;
		let detection_engine = Arc::new(detection_engine);
		self.detection_engine = Some(Arc::clone(&detection_engine));
		info!("Detection engine initialized successfully");

		// Create channels for packet flow
		let (packet_sender, packet_receiver) = mpsc::channel::<PacketData>(10000);
		info!("Created packet channel with capacity 10000");

		let shutdown_token = self.shutdown_token.clone();

		// Determine capture mode
		let use_simulation = self.config.use_simulation || self.should_use_simulation();
		info!("Capture mode determined: simulation={}", use_simulation);

		// Start appropriate capture task
		let capture_handle = if use_simulation {
			info!("Starting SIMULATED packet capture");
			let stats = Arc::clone(&self.stats);
			let capture_shutdown = shutdown_token.clone();

			// Log initial stats
			debug!("Stats before simulation start: {:?}", stats.read());

			tokio::spawn(async move {
				info!("Simulated capture task spawned");
				tokio::select! {
					result = capture::SimulatedCapture::generate_packets(packet_sender, stats) => {
						match result {
							Ok(_) => info!("Simulated capture completed normally"),
							Err(e) => error!("Simulated capture failed: {}", e),
						}
					}
					_ = capture_shutdown.cancelled() => {
						info!("Simulated capture shutting down via cancellation token");
					}
				}
				info!("Simulated capture task exiting");
			})
		} else {
			// Try real packet capture
			match capture::PacketCapture::new(&self.config) {
				Ok(mut packet_capture) => {
					info!("Starting REAL packet capture");
					let stats = Arc::clone(&self.stats);
					let capture_shutdown = shutdown_token.clone();
					tokio::spawn(async move {
						info!("Real capture task spawned");
						tokio::select! {
							result = packet_capture.start_capture(packet_sender, stats) => {
								match result {
									Ok(_) => info!("Packet capture completed normally"),
									Err(e) => error!("Packet capture failed: {}", e),
								}
							}
							_ = capture_shutdown.cancelled() => {
								info!("Packet capture shutting down via cancellation token");
							}
						}
						info!("Real capture task exiting");
					})
				}
				Err(e) => {
					warn!("Failed to initialize packet capture: {}, falling back to simulation mode", e);
					// Fall back to simulation
					let stats = Arc::clone(&self.stats);
					let capture_shutdown = shutdown_token.clone();
					tokio::spawn(async move {
						info!("Fallback simulated capture task spawned");
						tokio::select! {
							result = capture::SimulatedCapture::generate_packets(packet_sender, stats) => {
								match result {
									Ok(_) => info!("Fallback simulated capture completed normally"),
									Err(e) => error!("Fallback simulated capture failed: {}", e),
								}
							}
							_ = capture_shutdown.cancelled() => {
								info!("Fallback simulated capture shutting down");
							}
						}
						info!("Fallback simulated capture task exiting");
					})
				}
			}
		};

		// Start detection task
		info!("Starting detection task...");
		let detection_handle = {
			let detection_engine = Arc::clone(&detection_engine);
			let stats = Arc::clone(&self.stats);
			let detection_shutdown = shutdown_token.clone();

			tokio::spawn(async move {
				info!("Detection task spawned");
				tokio::select! {
					result = detection_engine.process_packets(packet_receiver, stats) => {
						match result {
							Ok(_) => info!("Detection engine completed normally"),
							Err(e) => error!("Detection engine failed: {}", e),
						}
					}
					_ = detection_shutdown.cancelled() => {
						info!("Detection engine shutting down via cancellation token");
					}
				}
				info!("Detection task exiting");
			})
		};

		info!("Network IDS system started successfully - all tasks spawned");

		// Periodic stats monitor (logs only)
		let stats_monitor = Arc::clone(&self.stats);
		let monitor_shutdown = shutdown_token.clone();
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
			loop {
				tokio::select! {
					_ = interval.tick() => {
						let stats = stats_monitor.read();
						info!(
							"STATS UPDATE: packets={}, bytes={}, threats={}, rate={:.2}, flows={}",
							stats.packets_processed,
							stats.bytes_processed,
							stats.threats_detected,
							stats.processing_rate,
							stats.active_flows
						);
					}
					_ = monitor_shutdown.cancelled() => {
						info!("Stats monitor shutting down");
						break;
					}
				}
			}
		});
		
		// === System stats updater (per-process CPU) ===
		let sys_stats = Arc::clone(&self.stats);
		let sys_updater_shutdown = shutdown_token.clone();
		tokio::spawn(async move {
			use std::time::Duration;

			// We sample the current process using sysinfo.
			let mut sys = System::new_all();
			// We’ll resolve our PID once and then refresh the process each tick.
			let pid = sysinfo::get_current_pid().expect("failed to get current pid");

			// Prime sysinfo so the second read has deltas.
			sys.refresh_process(pid);

			let mut interval = tokio::time::interval(Duration::from_secs(2));
			loop {
				tokio::select! {
					_ = interval.tick() => {
						// Refresh this process and global memory pool.
						sys.refresh_process(pid);
						sys.refresh_memory();

						// Per-process CPU percent (relative to one core; can exceed 100 on multicore).
						let cpu = sys.process(pid)
							.map(|p| p.cpu_usage())
							.unwrap_or(0.0);

						// We'll keep memory as system memory used (global), as before.
						let used_mem_bytes = sys.used_memory() * 1024;

						// Write into the shared SystemStats.
						let mut s = sys_stats.write();
						s.cpu_usage = cpu;
						s.memory_usage = used_mem_bytes;
					}
					_ = sys_updater_shutdown.cancelled() => {
						info!("System stats updater shutting down");
						break;
					}
				}
			}
		});


		// Detach handles to avoid unused warnings; tasks are supervised by the token.
		let _ = capture_handle;
		let _ = detection_handle;

		// IMPORTANT CHANGE: do NOT wait for shutdown here.
		// Return immediately so the outer Mutex is released.
		Ok(())
	}

    
    /// Check if simulation should be used
    fn should_use_simulation(&self) -> bool {
        // Check if we're on Windows without proper pcap setup
        #[cfg(target_os = "windows")]
        {
            info!("Platform: Windows - defaulting to simulation mode");
            false
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Platform: Non-Windows - attempting real packet capture");
            false
        }
    }
    
    /// Shutdown the IDS system
    pub fn shutdown(&self) {
        info!("Shutdown requested");
        debug!("Final stats: {:?}", self.stats.read());
        self.shutdown_token.cancel();
        info!("Cancellation token triggered");
    }
    
    /// Get system statistics
    pub fn get_stats(&self) -> SystemStats {
        let stats = self.stats.read().clone();
        debug!("Getting stats: packets={}, bytes={}, rate={:.2}",
            stats.packets_processed,
            stats.bytes_processed,
            stats.processing_rate
        );
        stats
    }
    
    /// Subscribe to threat alerts
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<ThreatAlert> {
        info!("New alert subscription created");
        self.alert_sender.subscribe()
    }
    
    /// Get recent alerts from the detection engine
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<ThreatAlert> {
        debug!("Getting recent alerts with limit: {}", limit);
        if let Some(engine) = &self.detection_engine {
            let alerts = engine.get_recent_alerts(limit);
            debug!("Retrieved {} alerts", alerts.len());
            alerts
        } else {
            warn!("Detection engine not initialized, returning empty alerts");
            Vec::new()
        }
    }
	
	/// Get reference to the detection engine
	pub fn get_detection_engine(&self) -> Option<&Arc<detection::DetectionEngine>> {
		self.detection_engine.as_ref()
	}
}