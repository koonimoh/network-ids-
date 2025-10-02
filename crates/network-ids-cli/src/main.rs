//! Interactive command-line interface for Network IDS

use anyhow::Result;
use clap::Parser;
use network_ids_core::{NetworkIDS, types::{SystemConfig, Severity}};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{info, error, Level};
use std::sync::Arc;
use tokio::sync::Mutex;
use colored::*;

#[derive(Parser)]
#[command(name = "network-ids")]
#[command(about = "ML-powered Network Intrusion Detection System - Interactive CLI")]
#[command(version = "1.0.0")]
struct Cli {
    /// Start in non-interactive mode
    #[arg(long)]
    no_interactive: bool,
}

struct IDSSession {
    ids: Option<Arc<Mutex<NetworkIDS>>>,
    running: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_level(false)
        .init();

    let _cli = Cli::parse();
    
    // Print welcome banner
    print_banner();
    
    // Create session
    let session = Arc::new(Mutex::new(IDSSession {
        ids: None,
        running: false,
    }));
    
    // Start interactive shell
    run_interactive_shell(session).await
}

fn print_banner() {
    println!("{}", "╔════════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║     Network Intrusion Detection System - Interactive CLI      ║".bright_cyan());
    println!("{}", "║                    ML-Powered Threat Detection                 ║".bright_cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
    println!("{}", "Type 'help' for available commands, 'exit' to quit".bright_black());
    println!();
}

async fn run_interactive_shell(session: Arc<Mutex<IDSSession>>) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    
    loop {
        // Print prompt
        let running = session.lock().await.running;
        let prompt = if running {
            format!("{} ", "ids>".bright_green().bold())
        } else {
            format!("{} ", "ids>".bright_red().bold())
        };
        
        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush()?;
        
        // Read input
        let line = match reader.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(e) => {
                error!("Failed to read line: {}", e);
                continue;
            }
        };
        
        let command = line.trim();
        if command.is_empty() {
            continue;
        }
        
        // Handle command
        match handle_command(command, Arc::clone(&session)).await {
            Ok(should_exit) => {
                if should_exit {
                    break;
                }
            }
            Err(e) => {
                println!("{} {}", "Error:".bright_red().bold(), e);
            }
        }
    }
    
    // Cleanup on exit
    println!("\n{}", "Shutting down...".yellow());
    let mut sess = session.lock().await;
    if let Some(ids) = &sess.ids {
        let ids_locked = ids.lock().await;
        ids_locked.shutdown();
    }
    
    println!("{}", "Goodbye!".bright_green());
    Ok(())
}

async fn handle_command(command: &str, session: Arc<Mutex<IDSSession>>) -> Result<bool> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(false);
    }
    
    match parts[0] {
        "help" | "h" | "?" => {
            print_help();
        }
        
        "start" => {
            start_ids(session, &parts[1..]).await?;
        }
        
        "stop" => {
            stop_ids(session).await?;
        }
        
        "status" | "s" => {
            show_status(session).await?;
        }
        
        "stats" => {
            show_stats(session, &parts[1..]).await?;
        }
        
        "alerts" => {
            show_alerts(session, &parts[1..]).await?;
        }
        
        "ai" => {
            if parts.len() < 2 {
                println!("{}", "Usage: ai <query>".yellow());
                println!("Example: ai what are the top threats?");
            } else {
                let query = parts[1..].join(" ");
                query_ai(session, &query).await?;
            }
        }
        
        "clear" | "cls" => {
            print!("\x1B[2J\x1B[1;1H");
            print_banner();
        }
        
        "exit" | "quit" | "q" => {
            return Ok(true);
        }
        
        _ => {
            println!("{} Unknown command: '{}'", "Error:".bright_red().bold(), parts[0]);
            println!("Type 'help' for available commands");
        }
    }
    
    Ok(false)
}

fn print_help() {
    println!("\n{}", "Available Commands:".bright_cyan().bold());
    println!();
    println!("  {}              Start the IDS system", "start".bright_green());
    println!("                       Options: --simulate (use simulated traffic)");
    println!();
    println!("  {}               Stop the IDS system", "stop".bright_green());
    println!();
    println!("  {}             Show system status", "status".bright_green());
    println!();
    println!("  {}              Show system statistics", "stats".bright_green());
    println!("                       Options: --live (continuous updates)");
    println!("                                --protocols (protocol distribution)");
    println!("                                --threats (threat breakdown)");
    println!();
    println!("  {}             Show recent alerts", "alerts".bright_green());
    println!("                       Options: --limit <n> (show n alerts)");
    println!("                                --critical (only critical)");
    println!("                                --high (high and above)");
    println!();
    println!("  {} <query>       Query AI about your data", "ai".bright_green());
    println!("                       Example: ai what are the top 3 threats?");
    println!();
    println!("  {}              Clear screen", "clear".bright_green());
    println!();
    println!("  {}               Exit the CLI", "exit".bright_green());
    println!();
}

async fn start_ids(session: Arc<Mutex<IDSSession>>, args: &[&str]) -> Result<()> {
    let mut sess = session.lock().await;
    
    if sess.running {
        println!("{}", "IDS is already running!".yellow());
        return Ok(());
    }
    
    // Parse options
    let simulate = args.contains(&"--simulate");
    
    println!("{}", "Starting IDS...".bright_cyan());
    
    let mut config = SystemConfig::default();
    config.use_simulation = simulate;
    
    let mut ids = NetworkIDS::new(config)?;
    
    // Subscribe to alerts before starting
    let alert_receiver = ids.subscribe_alerts();
    
    // Spawn alert handler
    tokio::spawn(async move {
        handle_alerts(alert_receiver).await;
    });
    
    // Start IDS
    ids.start().await?;
    
    sess.ids = Some(Arc::new(Mutex::new(ids)));
    sess.running = true;
    
    println!("{}", "✓ IDS started successfully".bright_green());
    if simulate {
        println!("{}", "  Mode: Simulation".bright_black());
    }
    
    Ok(())
}

async fn handle_alerts(mut receiver: tokio::sync::broadcast::Receiver<network_ids_core::types::ThreatAlert>) {
    while let Ok(alert) = receiver.recv().await {
        let severity_color = match alert.severity {
            Severity::Critical => "red",
            Severity::High => "yellow",
            Severity::Medium => "blue",
            Severity::Low => "white",
        };
        
        let severity_str = format!("{}", alert.severity).color(severity_color).bold();
        println!("\n{} {} {} from {}",
                 "🚨".bright_red(),
                 severity_str,
                 alert.threat_type.to_string().bright_white().bold(),
                 alert.source_ip.to_string().bright_cyan());
        println!("   {}", alert.description.bright_black());
        print!("\nids> ");
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
}

async fn stop_ids(session: Arc<Mutex<IDSSession>>) -> Result<()> {
    let mut sess = session.lock().await;
    
    if !sess.running {
        println!("{}", "IDS is not running".yellow());
        return Ok(());
    }
    
    println!("{}", "Stopping IDS...".bright_cyan());
    
    if let Some(ids) = &sess.ids {
        let ids_locked = ids.lock().await;
        ids_locked.shutdown();
    }
    
    sess.ids = None;
    sess.running = false;
    
    println!("{}", "✓ IDS stopped".bright_green());
    
    Ok(())
}

async fn show_status(session: Arc<Mutex<IDSSession>>) -> Result<()> {
    let sess = session.lock().await;
    
    println!("\n{}", "System Status:".bright_cyan().bold());
    println!("{}", "═".repeat(50).bright_black());
    
    if sess.running {
        println!("Status: {}", "Running".bright_green().bold());
        
        if let Some(ids) = &sess.ids {
            let ids_locked = ids.lock().await;
            let stats = ids_locked.get_stats();
            
            println!("Uptime: {} seconds", 
                     (chrono::Utc::now() - stats.start_time).num_seconds().to_string().bright_white());
            println!("Packets: {}", stats.packets_processed.to_string().bright_white());
            println!("Threats: {}", stats.threats_detected.to_string().bright_red());
        }
    } else {
        println!("Status: {}", "Stopped".bright_red().bold());
    }
    
    println!("Version: {}", env!("CARGO_PKG_VERSION").bright_white());
    println!();
    
    Ok(())
}

async fn show_stats(session: Arc<Mutex<IDSSession>>, args: &[&str]) -> Result<()> {
    let sess = session.lock().await;
    
    if !sess.running {
        println!("{}", "IDS is not running. Start it with 'start'".yellow());
        return Ok(());
    }
    
    let ids = sess.ids.as_ref().ok_or_else(|| anyhow::anyhow!("No IDS instance"))?;
    let ids_locked = ids.lock().await;
    let stats = ids_locked.get_stats();
    
    let live = args.contains(&"--live");
    let show_protocols = args.contains(&"--protocols");
    let show_threats = args.contains(&"--threats");
    
    if live {
        println!("{}", "Live stats (Ctrl+C to stop):".bright_cyan().bold());
        println!();
        
        // Live update loop
        drop(ids_locked);
        drop(sess);
        
        loop {
            let sess = session.lock().await;
            if !sess.running {
                break;
            }
            
            if let Some(ids) = &sess.ids {
                let ids_locked = ids.lock().await;
                let stats = ids_locked.get_stats();
                
                print!("\r{} Packets: {} | Threats: {} | Rate: {:.2} pps   ",
                       "📊".to_string(),
                       stats.packets_processed.to_string().bright_white(),
                       stats.threats_detected.to_string().bright_red(),
                       stats.processing_rate.to_string().bright_green());
                
                use std::io::Write;
                std::io::stdout().flush()?;
            }
            
            drop(sess);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        println!();
    } else {
        // Static stats
        println!("\n{}", "System Statistics:".bright_cyan().bold());
        println!("{}", "═".repeat(50).bright_black());
        
        println!("Packets Processed: {}", stats.packets_processed.to_string().bright_white());
        println!("Bytes Processed:   {}", format_bytes(stats.bytes_processed).bright_white());
        println!("Threats Detected:  {}", stats.threats_detected.to_string().bright_red());
        println!("Active Flows:      {}", stats.active_flows.to_string().bright_white());
        println!("Processing Rate:   {} pps", format!("{:.2}", stats.processing_rate).bright_green());
        println!("CPU Usage:         {}%", format!("{:.1}", stats.cpu_usage).bright_yellow());
        println!("Memory Usage:      {}", format_bytes(stats.memory_usage).bright_yellow());
        
        if show_protocols {
            println!("\n{}", "Protocol Distribution:".bright_cyan());
            for (protocol, count) in &stats.protocol_distribution {
                println!("  {}: {}", protocol.to_string().bright_white(), count.to_string().bright_black());
            }
        }
        
        if show_threats {
            println!("\n{}", "Threat Breakdown:".bright_cyan());
            for (severity, count) in &stats.alert_counts {
                let color = match severity {
                    Severity::Critical => "red",
                    Severity::High => "yellow",
                    Severity::Medium => "blue",
                    Severity::Low => "white",
                };
                println!("  {}: {}", severity.to_string().color(color), count.to_string().bright_black());
            }
        }
        
        println!();
    }
    
    Ok(())
}

async fn show_alerts(session: Arc<Mutex<IDSSession>>, args: &[&str]) -> Result<()> {
    let sess = session.lock().await;
    
    if !sess.running {
        println!("{}", "IDS is not running. Start it with 'start'".yellow());
        return Ok(());
    }
    
    let ids = sess.ids.as_ref().ok_or_else(|| anyhow::anyhow!("No IDS instance"))?;
    let ids_locked = ids.lock().await;
    
    // Parse options
    let mut limit = 10;
    let mut filter_severity: Option<Severity> = None;
    
    for (i, arg) in args.iter().enumerate() {
        match *arg {
            "--limit" => {
                if let Some(n) = args.get(i + 1) {
                    limit = n.parse().unwrap_or(10);
                }
            }
            "--critical" => filter_severity = Some(Severity::Critical),
            "--high" => filter_severity = Some(Severity::High),
            _ => {}
        }
    }
    
    let all_alerts = ids_locked.get_recent_alerts(100);
    let filtered_alerts: Vec<_> = if let Some(min_severity) = filter_severity {
        all_alerts.into_iter()
            .filter(|a| a.severity >= min_severity)
            .take(limit)
            .collect()
    } else {
        all_alerts.into_iter().take(limit).collect()
    };
    
    if filtered_alerts.is_empty() {
        println!("{}", "No alerts to display".bright_black());
        return Ok(());
    }
    
    println!("\n{} (showing {})", "Recent Alerts:".bright_cyan().bold(), filtered_alerts.len());
    println!("{}", "═".repeat(70).bright_black());
    
    for (i, alert) in filtered_alerts.iter().enumerate() {
        let severity_color = match alert.severity {
            Severity::Critical => "red",
            Severity::High => "yellow",
            Severity::Medium => "blue",
            Severity::Low => "white",
        };
        
        println!("\n{} {} {}",
                 format!("{}.", i + 1).bright_black(),
                 alert.severity.to_string().color(severity_color).bold(),
                 alert.threat_type.to_string().bright_white().bold());
        println!("   From: {} → {}", 
                 alert.source_ip.to_string().bright_cyan(),
                 alert.target_ip.map(|ip| ip.to_string()).unwrap_or_else(|| "N/A".to_string()).bright_cyan());
        println!("   {}", alert.description.bright_black());
        println!("   Confidence: {}%", (alert.confidence * 100.0).round().to_string().bright_green());
    }
    
    println!();
    
    Ok(())
}

async fn query_ai(session: Arc<Mutex<IDSSession>>, query: &str) -> Result<()> {
    let sess = session.lock().await;
    
    if !sess.running {
        println!("{}", "IDS is not running. Start it first with 'start'".yellow());
        return Ok(());
    }
    
    // Check for API keys
    let provider = if std::env::var("OPENAI_API_KEY").is_ok_and(|k| !k.is_empty()) {
        "openai"
    } else if std::env::var("ANTHROPIC_API_KEY").is_ok_and(|k| !k.is_empty()) {
        "anthropic"
    } else if std::env::var("GEMINI_API_KEY").is_ok_and(|k| !k.is_empty()) {
        "gemini"
    } else {
        println!("{}", "No AI provider configured. Set one of:".yellow());
        println!("  - OPENAI_API_KEY");
        println!("  - ANTHROPIC_API_KEY");
        println!("  - GEMINI_API_KEY");
        return Ok(());
    };
    
    println!("{}", format!("Querying {} AI...", provider).bright_cyan());
    
    // Build request
    let ids = sess.ids.as_ref().unwrap();
    let ids_locked = ids.lock().await;
    let stats = ids_locked.get_stats();
    let alerts = ids_locked.get_recent_alerts(50);
    
    let context = build_ai_context(&stats, &alerts);
    drop(ids_locked);
    drop(sess);
    
    // Make API request
    let client = reqwest::Client::new();
    let api_key = std::env::var(format!("{}_API_KEY", provider.to_uppercase()))?;
    
    let response_text = match provider {
        "openai" => {
            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&serde_json::json!({
                    "model": "gpt-4o",
                    "messages": [
                        {"role": "system", "content": context},
                        {"role": "user", "content": query}
                    ],
                    "max_tokens": 1000
                }))
                .send()
                .await?;
            
            let data: serde_json::Value = response.json().await?;
            data["choices"][0]["message"]["content"].as_str().unwrap_or("No response").to_string()
        }
        "anthropic" => {
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&serde_json::json!({
                    "model": "claude-sonnet-4-20250514",
                    "max_tokens": 2000,
                    "system": context,
                    "messages": [{"role": "user", "content": query}]
                }))
                .send()
                .await?;
            
            let data: serde_json::Value = response.json().await?;
            data["content"][0]["text"].as_str().unwrap_or("No response").to_string()
        }
        "gemini" => {
            let prompt = format!("{}\n\nUser: {}", context, query);
            let response = client
                .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key))
                .json(&serde_json::json!({"contents": [{"parts": [{"text": prompt}]}]}))
                .send()
                .await?;
            
            let data: serde_json::Value = response.json().await?;
            data["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("No response").to_string()
        }
        _ => unreachable!()
    };
    
    println!("\n{}", "AI Response:".bright_cyan().bold());
    println!("{}", "─".repeat(70).bright_black());
    println!("{}", response_text);
    println!();
    
    Ok(())
}

fn build_ai_context(stats: &network_ids_core::types::SystemStats, alerts: &[network_ids_core::types::ThreatAlert]) -> String {
    format!(
        "You are a cybersecurity analyst. System stats: {} packets, {} threats, {} active flows. Recent alerts: {}",
        stats.packets_processed,
        stats.threats_detected,
        stats.active_flows,
        alerts.len()
    )
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit])
}