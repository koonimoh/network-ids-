//! REST API server for Network IDS

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, delete},
    Json, Router,
};
use network_ids_core::{NetworkIDS, types::*};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock, Mutex};
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, error, Level};
use chrono::{DateTime, Utc};  
use std::collections::HashMap; 

/// Application state with proper task handle management
#[derive(Clone)]
struct AppState {
    ids: Arc<RwLock<Option<Arc<Mutex<NetworkIDS>>>>>,
    ids_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    alert_receiver: Arc<RwLock<Option<broadcast::Receiver<ThreatAlert>>>>,
    blocklist: Arc<RwLock<HashMap<String, BlockedIP>>>, 
}

/// Blocked IP entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockedIP {
    ip: String,
    reason: String,
    blocked_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    notes: Option<String>,
}

/// Query parameters for alerts endpoint
#[derive(Debug, Deserialize)]
struct AlertsQuery {
    limit: Option<usize>,
}

/// System status response
#[derive(Debug, Serialize)]
struct SystemStatus {
    running: bool,
    uptime_seconds: u64,
    version: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	
	// Load env
	dotenv::dotenv().ok();
	// Debug: Check if API key is loaded
	match std::env::var("ABUSEIPDB_API_KEY") {
		Ok(key) => {
			if key.is_empty() {
				println!("⚠️  ABUSEIPDB_API_KEY is EMPTY");
			} else {
				println!("✓ ABUSEIPDB_API_KEY loaded (length: {})", key.len());
			}
		}
		Err(_) => {
			println!("✗ ABUSEIPDB_API_KEY NOT FOUND");
		}
	}
	
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Network IDS API Server");

    // Initialize application state
    let app_state = AppState {
        ids: Arc::new(RwLock::new(None)),
        ids_task: Arc::new(RwLock::new(None)),
        alert_receiver: Arc::new(RwLock::new(None)),
		blocklist: Arc::new(RwLock::new(HashMap::new())),
    };

    // Build router
    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/status", get(get_status))
        .route("/api/stats", get(get_stats))
        .route("/api/alerts", get(get_alerts))
        .route("/api/start", post(start_ids))
        .route("/api/stop", post(stop_ids))
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
		.route("/api/ip-lookup/:ip", get(lookup_ip))
		.route("/api/blocklist", get(get_blocklist))      
		.route("/api/blocklist", post(add_to_blocklist)) 
		.route("/api/blocklist/:ip", delete(remove_from_blocklist))
		.route("/api/geolocation", get(get_threat_geolocation))
		.route("/api/flows", get(get_active_flows))
        .route("/ws/alerts", get(websocket_alerts))
		.route("/api/ai/query", post(handle_ai_query))
        .nest_service("/assets", ServeDir::new("web/dist/assets"))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("API server listening on http://{}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await?;

	// Create shutdown channel
	let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

	// Spawn stdin reader for graceful shutdown
	tokio::spawn(async move {
		use tokio::io::{AsyncBufReadExt, BufReader};
		let stdin = tokio::io::stdin();
		let mut reader = BufReader::new(stdin).lines();
		
		println!("\n💡 Type 'exit' or 'bye' to shutdown gracefully, or press Ctrl+C\n");
		
		while let Ok(Some(line)) = reader.next_line().await {
			let cmd = line.trim().to_lowercase();
			if cmd == "exit" || cmd == "bye" {
				println!("👋 Shutting down gracefully...");
				let _ = shutdown_tx.send(()).await;
				break;
			}
		}
	});

	// Start server with graceful shutdown
	let server = axum::serve(listener, app)
		.with_graceful_shutdown(async move {
			let _ = shutdown_rx.recv().await;
			println!("✅ Server shutdown complete");
		});

	server.await?;

	Ok(())
}

/// Serve the main dashboard HTML
async fn serve_dashboard() -> impl IntoResponse {
    let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Network IDS Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f0f2f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { text-align: center; margin-bottom: 40px; }
        .card { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); margin-bottom: 20px; }
        .status { display: inline-block; padding: 5px 15px; border-radius: 20px; color: white; font-weight: bold; }
        .status.running { background: #28a745; }
        .status.stopped { background: #dc3545; }
        button { background: #007bff; color: white; border: none; padding: 10px 20px; border-radius: 5px; cursor: pointer; margin-right: 10px; }
        button:hover { background: #0056b3; }
        button:disabled { background: #6c757d; cursor: not-allowed; }
        .alert { padding: 10px; margin: 5px 0; border-radius: 5px; }
        .alert.high { background: #f8d7da; border: 1px solid #f5c6cb; }
        .alert.medium { background: #fff3cd; border: 1px solid #ffeaa7; }
        .alert.low { background: #d1ecf1; border: 1px solid #bee5eb; }
        .loading { opacity: 0.6; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Network Intrusion Detection System</h1>
            <p>Real-time network monitoring and threat detection</p>
        </div>
        
        <div class="card">
            <h2>System Status</h2>
            <p>Status: <span class="status stopped" id="status">Stopped</span></p>
            <p>Version: <span id="version">0.1.0</span></p>
            <button id="startBtn" onclick="startIDS()">Start IDS</button>
            <button id="stopBtn" onclick="stopIDS()" disabled>Stop IDS</button>
        </div>
        
        <div class="card">
            <h2>System Statistics</h2>
            <div id="stats">
                <p>Packets Processed: <span id="packets">0</span></p>
                <p>Bytes Processed: <span id="bytes">0</span></p>
                <p>Threats Detected: <span id="threats">0</span></p>
                <p>Active Flows: <span id="flows">0</span></p>
                <p>Processing Rate: <span id="rate">0</span> packets/sec</p>
            </div>
        </div>
        
        <div class="card">
            <h2>Recent Alerts</h2>
            <div id="alerts">
                <p>No alerts detected</p>
            </div>
        </div>
    </div>

    <script>
        let updateInterval = null;
        let isRunning = false;

        async function startIDS() {
            const btn = document.getElementById('startBtn');
            const stopBtn = document.getElementById('stopBtn');
            btn.disabled = true;
            btn.textContent = 'Starting...';
            
            try {
                const response = await fetch('/api/start', { method: 'POST' });
                const result = await response.json();
                if (result.success) {
                    isRunning = true;
                    document.getElementById('status').textContent = 'Running';
                    document.getElementById('status').className = 'status running';
                    btn.textContent = 'Start IDS';
                    btn.disabled = true;
                    stopBtn.disabled = false;
                    startUpdateLoop();
                } else {
                    alert('Failed to start IDS: ' + (result.error || 'Unknown error'));
                    btn.disabled = false;
                    btn.textContent = 'Start IDS';
                }
            } catch (error) {
                console.error('Error starting IDS:', error);
                alert('Error starting IDS');
                btn.disabled = false;
                btn.textContent = 'Start IDS';
            }
        }

        async function stopIDS() {
            const btn = document.getElementById('stopBtn');
            const startBtn = document.getElementById('startBtn');
            btn.disabled = true;
            btn.textContent = 'Stopping...';
            
            try {
                const response = await fetch('/api/stop', { method: 'POST' });
                const result = await response.json();
                if (result.success) {
                    isRunning = false;
                    document.getElementById('status').textContent = 'Stopped';
                    document.getElementById('status').className = 'status stopped';
                    btn.textContent = 'Stop IDS';
                    btn.disabled = true;
                    startBtn.disabled = false;
                    stopUpdateLoop();
                } else {
                    alert('Failed to stop IDS: ' + (result.error || 'Unknown error'));
                    btn.disabled = false;
                    btn.textContent = 'Stop IDS';
                }
            } catch (error) {
                console.error('Error stopping IDS:', error);
                alert('Error stopping IDS');
                btn.disabled = false;
                btn.textContent = 'Stop IDS';
            }
        }

        async function updateStats() {
            try {
                const response = await fetch('/api/stats');
                const result = await response.json();
                if (result.success && result.data) {
                    const stats = result.data;
                    document.getElementById('packets').textContent = stats.packets_processed || 0;
                    document.getElementById('bytes').textContent = formatBytes(stats.bytes_processed || 0);
                    document.getElementById('threats').textContent = stats.threats_detected || 0;
                    document.getElementById('flows').textContent = stats.active_flows || 0;
                    document.getElementById('rate').textContent = (stats.processing_rate || 0).toFixed(2);
                }
            } catch (error) {
                console.error('Error fetching stats:', error);
            }
        }

        async function updateStatus() {
            try {
                const response = await fetch('/api/status');
                const result = await response.json();
                if (result.success && result.data) {
                    const status = result.data;
                    const running = status.running;
                    
                    document.getElementById('status').textContent = running ? 'Running' : 'Stopped';
                    document.getElementById('status').className = running ? 'status running' : 'status stopped';
                    document.getElementById('version').textContent = status.version;
                    
                    const startBtn = document.getElementById('startBtn');
                    const stopBtn = document.getElementById('stopBtn');
                    startBtn.disabled = running;
                    stopBtn.disabled = !running;
                    
                    if (running !== isRunning) {
                        isRunning = running;
                        if (running) {
                            startUpdateLoop();
                        } else {
                            stopUpdateLoop();
                        }
                    }
                }
            } catch (error) {
                console.error('Error fetching status:', error);
            }
        }

        async function updateAlerts() {
            try {
                const response = await fetch('/api/alerts?limit=10');
                const result = await response.json();
                if (result.success && result.data && result.data.length > 0) {
                    const alertsDiv = document.getElementById('alerts');
                    alertsDiv.innerHTML = result.data.map(alert => 
                        `<div class="alert ${alert.severity.toLowerCase()}">
                            <strong>${alert.threat_type}</strong> - ${alert.description}
                            <br><small>${new Date(alert.timestamp).toLocaleString()}</small>
                        </div>`
                    ).join('');
                }
            } catch (error) {
                console.error('Error fetching alerts:', error);
            }
        }

        function formatBytes(bytes) {
            const units = ['B', 'KB', 'MB', 'GB', 'TB'];
            let size = bytes;
            let unitIndex = 0;
            while (size >= 1024 && unitIndex < units.length - 1) {
                size /= 1024;
                unitIndex++;
            }
            return size.toFixed(2) + ' ' + units[unitIndex];
        }

        function startUpdateLoop() {
            if (updateInterval) clearInterval(updateInterval);
            updateInterval = setInterval(() => {
                if (isRunning) {
                    updateStats();
                    updateAlerts();
                }
            }, 1000);
        }

        function stopUpdateLoop() {
            if (updateInterval) {
                clearInterval(updateInterval);
                updateInterval = null;
            }
        }

        // Initial update
        updateStatus();
        updateStats();
        
        // Update status every 2 seconds
        setInterval(updateStatus, 2000);
    </script>
</body>
</html>
    "#;
    
    Html(html_content)
}

/// Get system status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let ids_guard = state.ids.read().await;
    let running = ids_guard.is_some();
    
    let status = SystemStatus {
        running,
        uptime_seconds: if running { 3600 } else { 0 },
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(ApiResponse::success(status))
}

/// Get system statistics with proper error handling
// main.rs — replace the entire get_stats function
/// Get system statistics with proper error handling
async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let ids_guard = state.ids.read().await;

    if let Some(ids_arc) = ids_guard.as_ref() {
        // Block briefly for the lock instead of returning zeros.
        let ids = ids_arc.lock().await;
        let stats = ids.get_stats();
        Json(ApiResponse::success(stats))
    } else {
        Json(ApiResponse::success(SystemStats::new()))
    }
}


/// Get threat alerts
// main.rs — replace get_alerts to use a real lock instead of try_lock
/// Get threat alerts
async fn get_alerts(
    Query(params): Query<AlertsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ids_guard = state.ids.read().await;

    if let Some(ids_arc) = ids_guard.as_ref() {
        let ids = ids_arc.lock().await;
        let alerts = ids.get_recent_alerts(params.limit.unwrap_or(50));
        Json(ApiResponse::success(alerts))
    } else {
        Json(ApiResponse::success(Vec::<ThreatAlert>::new()))
    }
}


/// Start the IDS system with proper task management
async fn start_ids(State(state): State<AppState>) -> impl IntoResponse {
    // Check if already running
    {
        let ids_guard = state.ids.read().await;
        if ids_guard.is_some() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("IDS already running")),
            );
        }
    }

    // Create config with simulation mode for testing
    let mut config = SystemConfig::default();
    config.use_simulation = true; // Enable simulation mode if real capture fails
    
    match NetworkIDS::new(config) {
        Ok(mut ids) => {
            // Subscribe to alerts before starting
            let alert_receiver = ids.subscribe_alerts();
            
            // Store alert receiver
            {
                let mut receiver_guard = state.alert_receiver.write().await;
                *receiver_guard = Some(alert_receiver);
            }
            
            // Create Arc for IDS
            let ids_arc = Arc::new(Mutex::new(ids));
            
            // Store IDS reference
            {
                let mut ids_guard = state.ids.write().await;
                *ids_guard = Some(Arc::clone(&ids_arc));
            }
            
            // Start IDS in background task
            let ids_for_task = Arc::clone(&ids_arc);
            let task_handle = tokio::spawn(async move {
                let mut ids_locked = ids_for_task.lock().await;
                if let Err(e) = ids_locked.start().await {
                    error!("IDS failed: {}", e);
                }
            });
            
            // Store task handle
            {
                let mut task_guard = state.ids_task.write().await;
                *task_guard = Some(task_handle);
            }
            
            info!("IDS started successfully");
            (StatusCode::OK, Json(ApiResponse::success("IDS started")))
        }
        Err(e) => {
            error!("Failed to create IDS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to start IDS: {}", e))),
            )
        }
    }
}

/// Stop the IDS system properly
async fn stop_ids(State(state): State<AppState>) -> impl IntoResponse {
    // Get and shutdown IDS
    let ids_arc = {
        let mut ids_guard = state.ids.write().await;
        ids_guard.take()
    };
    
    if let Some(ids_arc) = ids_arc {
        // Trigger shutdown
        {
            let ids = ids_arc.lock().await;
            ids.shutdown();
        }
        
        // Wait for task to complete
        {
            let mut task_guard = state.ids_task.write().await;
            if let Some(handle) = task_guard.take() {
                // Give it 5 seconds to shutdown gracefully
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    handle
                ).await;
            }
        }
        
        // Clear alert receiver
        {
            let mut receiver_guard = state.alert_receiver.write().await;
            *receiver_guard = None;
        }
        
        info!("IDS stopped successfully");
        (StatusCode::OK, Json(ApiResponse::success("IDS stopped")))
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("IDS not running")),
        )
    }
}

/// Get current configuration
async fn get_config(State(_state): State<AppState>) -> impl IntoResponse {
    let config = SystemConfig::default();
    Json(ApiResponse::success(config))
}

/// Update configuration
async fn update_config(
    State(_state): State<AppState>,
    Json(config): Json<SystemConfig>,
) -> impl IntoResponse {
    info!("Configuration updated: {:?}", config);
    Json(ApiResponse::success("Configuration updated"))
}

/// WebSocket endpoint for real-time alerts
async fn websocket_alerts(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket_alerts(socket, state))
}

/// Handle WebSocket connection for alerts
async fn handle_websocket_alerts(
    mut socket: axum::extract::ws::WebSocket,
    state: AppState,
) {
    use axum::extract::ws::Message;
    
    info!("WebSocket connection established for alerts");

    // Get alert receiver
    let mut receiver = {
        let receiver_guard = state.alert_receiver.read().await;
        if let Some(receiver) = receiver_guard.as_ref() {
            receiver.resubscribe()
        } else {
            let error_response: ApiResponse<String> = ApiResponse::error("IDS not running");
            let _ = socket.send(Message::Text(
                serde_json::to_string(&error_response).unwrap()
            )).await;
            return;
        }
    };

    // Handle incoming WebSocket messages and send alerts
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            
            // Send alerts when received
            alert = receiver.recv() => {
                match alert {
                    Ok(alert) => {
                        let message = serde_json::to_string(&ApiResponse::success(&alert)).unwrap();
                        if socket.send(Message::Text(message)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Alert channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                }
            }
        }
    }
}

/// Proxy endpoint for IP lookup to avoid CORS issues
async fn lookup_ip(
    axum::extract::Path(ip): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Get API key from environment variable
    let api_key = match std::env::var("ABUSEIPDB_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "success": false,
                    "error": "AbuseIPDB API key not configured"
                })),
            );
        }
    };

    // Make request to AbuseIPDB
    let url = format!(
        "https://api.abuseipdb.com/api/v2/check?ipAddress={}&maxAgeInDays=90",
        ip
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("Key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "success": true,
                            "data": data
                        })),
                    ),
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to parse response: {}", e)
                        })),
                    ),
                }
            } else {
                (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({
                        "success": false,
                        "error": "AbuseIPDB API request failed"
                    })),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Request failed: {}", e)
            })),
        ),
    }
}


/// Get all blocked IPs
async fn get_blocklist(State(state): State<AppState>) -> impl IntoResponse {
    let blocklist = state.blocklist.read().await;
    let entries: Vec<BlockedIP> = blocklist.values().cloned().collect();
    Json(serde_json::json!({
        "success": true,
        "data": entries
    }))
}

/// Add IP to blocklist
#[derive(Debug, Deserialize)]
struct BlockIPRequest {
    ip: String,
    reason: String,
    expires_in_hours: Option<i64>,
    notes: Option<String>,
}

async fn add_to_blocklist(
    State(state): State<AppState>,
    Json(request): Json<BlockIPRequest>,
) -> impl IntoResponse {
    // Validate IP format
    if request.ip.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "IP address cannot be empty"
            })),
        );
    }

    let mut blocklist = state.blocklist.write().await;
    
    let blocked_ip = BlockedIP {
        ip: request.ip.clone(),
        reason: request.reason,
        blocked_at: Utc::now(),
        expires_at: request.expires_in_hours.map(|hours| Utc::now() + chrono::Duration::hours(hours)),
        notes: request.notes,
    };

    blocklist.insert(request.ip.clone(), blocked_ip.clone());
    
    info!("IP {} added to blocklist", request.ip);
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": blocked_ip
        })),
    )
}

/// Remove IP from blocklist
async fn remove_from_blocklist(
    State(state): State<AppState>,
    axum::extract::Path(ip): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut blocklist = state.blocklist.write().await;
    
    if blocklist.remove(&ip).is_some() {
        info!("IP {} removed from blocklist", ip);
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": "IP removed from blocklist"
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "IP not found in blocklist"
            })),
        )
    }
}


/// Get active network flows
async fn get_active_flows(State(state): State<AppState>) -> impl IntoResponse {
    let ids_guard = state.ids.read().await;

    if let Some(ids_arc) = ids_guard.as_ref() {
        let ids = ids_arc.lock().await;
        
        // Get flows from detection engine if available
        if let Some(engine) = ids.get_detection_engine() {
            let flows = engine.get_active_flows();
            return Json(serde_json::json!({
                "success": true,
                "data": flows
            }));
        }
    }

    // Return empty if IDS not running
    Json(serde_json::json!({
        "success": true,
        "data": []
    }))
}



/// Geolocation response structures
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    #[serde(rename = "lat")]
    latitude: f64,
    #[serde(rename = "lon")]
    longitude: f64,
    country: String,
    city: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IpApiCoResponse {
    latitude: f64,
    longitude: f64,
    country_name: String,
    city: Option<String>,
}

/// Lookup geolocation using ip-api.com (primary, no key needed)
async fn lookup_geolocation_ipapi(ip: &str) -> Option<(f64, f64, String, Option<String>)> {
    let url = format!("http://ip-api.com/json/{}", ip);
    
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<IpApiResponse>().await {
                    return Some((data.latitude, data.longitude, data.country, data.city));
                }
            }
        }
        Err(_) => {}
    }
    None
}

/// Lookup geolocation using ipapi.co (fallback)
async fn lookup_geolocation_ipapico(ip: &str) -> Option<(f64, f64, String, Option<String>)> {
    let url = format!("https://ipapi.co/{}/json/", ip);
    
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<IpApiCoResponse>().await {
                    return Some((data.latitude, data.longitude, data.country_name, data.city));
                }
            }
        }
        Err(_) => {}
    }
    None
}

/// Get geolocation with fallback
async fn get_ip_geolocation(ip: &str) -> Option<(f64, f64, String, Option<String>)> {
    // Try primary API first
    if let Some(result) = lookup_geolocation_ipapi(ip).await {
        return Some(result);
    }
    
    // Fallback to secondary API
    if let Some(result) = lookup_geolocation_ipapico(ip).await {
        return Some(result);
    }
    
    None
}





/// Get geolocation data for recent threats
async fn get_threat_geolocation(State(state): State<AppState>) -> impl IntoResponse {
    let ids_guard = state.ids.read().await;

    if let Some(ids_arc) = ids_guard.as_ref() {
        let ids = ids_arc.lock().await;
        
        // Get recent alerts
        let alerts = ids.get_recent_alerts(50);
        
        // Extract unique source IPs
        let mut ip_data: HashMap<String, (String, u32, String)> = HashMap::new();
        
        for alert in alerts {
            let ip = alert.source_ip.to_string();
            let entry = ip_data.entry(ip.clone()).or_insert((ip, 0, "Low".to_string()));
            entry.1 += 1;
            
            // Update to highest severity
            let severity_order = |s: &str| match s {
                "Critical" => 4,
                "High" => 3,
                "Medium" => 2,
                _ => 1,
            };
            
            let alert_severity = alert.severity.to_string();
            if severity_order(&alert_severity) > severity_order(&entry.2) {
                entry.2 = alert_severity;
            }
        }
        
        // Build response with geolocation data
        let mut locations = Vec::new();
        
        for (ip, (_, count, severity)) in ip_data {
            // Skip private/local IPs
            if ip.starts_with("192.168.") || ip.starts_with("10.") || 
               ip.starts_with("172.") || ip == "127.0.0.1" {
                continue;
            }
            
            // Lookup geolocation
            if let Some((lat, lon, country, city)) = get_ip_geolocation(&ip).await {
                locations.push(serde_json::json!({
                    "ip": ip,
                    "latitude": lat,
                    "longitude": lon,
                    "country": country,
                    "city": city,
                    "count": count,
                    "severity": severity
                }));
            }
        }
        
        return Json(serde_json::json!({
            "success": true,
            "data": locations
        }));
    }

    Json(serde_json::json!({
        "success": true,
        "data": []
    }))
}



// ============================================================================
// AI Query Handler
// ============================================================================

/// Handle AI query request
async fn handle_ai_query(
    State(state): State<AppState>,
    Json(request): Json<AIQueryRequest>,
) -> impl IntoResponse {
    // Validate provider
    let provider = request.provider.to_lowercase();
    if !["openai", "anthropic", "gemini"].contains(&provider.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Invalid provider. Use: openai, anthropic, or gemini")),
        );
    }

    // Get API key from environment
    let api_key = match provider.as_str() {
        "openai" => std::env::var("OPENAI_API_KEY"),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY"),
        "gemini" => std::env::var("GEMINI_API_KEY"),
        _ => Err(std::env::VarError::NotPresent),
    };

    let api_key = match api_key {
        Ok(key) if !key.is_empty() => key,
        _ => {
            let var_name = match provider.as_str() {
                "openai" => "OPENAI_API_KEY",
                "anthropic" => "ANTHROPIC_API_KEY",
                "gemini" => "GEMINI_API_KEY",
                _ => "API_KEY",
            };
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::error(&format!(
                    "{} API key not configured. Set {} in .env file",
                    provider.to_uppercase(), var_name
                ))),
            );
        }
    };

    // Build context from IDS data
    let context = {
        let ids_guard = state.ids.read().await;
        if let Some(ids_arc) = ids_guard.as_ref() {
            let ids = ids_arc.lock().await;
            let stats = ids.get_stats();
            let alerts = ids.get_recent_alerts(50);
            build_ai_context(&stats, &alerts)
        } else {
            "IDS system is not running. No data available.".to_string()
        }
    };

    // Call appropriate AI provider
    let result = match provider.as_str() {
        "openai" => call_openai_api(&api_key, &request.query, &context, &request.conversation_history).await,
        "anthropic" => call_anthropic_api(&api_key, &request.query, &context, &request.conversation_history).await,
        "gemini" => call_gemini_api(&api_key, &request.query, &context).await,
        _ => Err("Invalid provider".to_string()),
    };

    match result {
        Ok(response) => (StatusCode::OK, Json(ApiResponse::success(response))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("AI request failed: {}", e))),
        ),
    }
}

/// Build context string from IDS data
fn build_ai_context(stats: &SystemStats, alerts: &[ThreatAlert]) -> String {
    let mut context = String::from(
        "You are an expert cybersecurity analyst for a Network Intrusion Detection System. \
        Provide clear, actionable security insights.\n\n"
    );

    // Add system stats
    context.push_str(&format!(
        "## System Statistics\n\
        - Packets processed: {}\n\
        - Bytes processed: {}\n\
        - Threats detected: {}\n\
        - Active flows: {}\n\
        - Processing rate: {:.2} pkt/s\n\n",
        stats.packets_processed,
        stats.bytes_processed,
        stats.threats_detected,
        stats.active_flows,
        stats.processing_rate
    ));

    // Add alert distribution
    if !stats.alert_counts.is_empty() {
        context.push_str("## Alert Distribution\n");
        for (severity, count) in &stats.alert_counts {
            context.push_str(&format!("- {}: {}\n", severity, count));
        }
        context.push('\n');
    }

    // Add recent alerts
    if !alerts.is_empty() {
        context.push_str(&format!("## Recent Alerts (showing {} of {})\n", alerts.len().min(10), alerts.len()));
        for (i, alert) in alerts.iter().take(10).enumerate() {
            context.push_str(&format!(
                "{}. [{}] {} from {} - {}\n",
                i + 1,
                alert.severity,
                alert.threat_type,
                alert.source_ip,
                alert.description
            ));
        }
    }

    context
}

/// Call OpenAI API
async fn call_openai_api(
    api_key: &str,
    query: &str,
    context: &str,
    history: &[ChatMessage],
) -> Result<AIQueryResponse, String> {
    let client = reqwest::Client::new();
    
    let mut messages = vec![serde_json::json!({"role": "system", "content": context})];
    
    for msg in history {
        messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
    }
    
    messages.push(serde_json::json!({"role": "user", "content": query}));

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4o",
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 2000
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("OpenAI API error {}: {}", status, error_text));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?
        .to_string();

    let tokens = data["usage"]["total_tokens"].as_u64().map(|t| t as u32);

    Ok(AIQueryResponse {
        response: content,
        model_used: "gpt-4o".to_string(),
        tokens_used: tokens,
    })
}

/// Call Anthropic API
async fn call_anthropic_api(
    api_key: &str,
    query: &str,
    context: &str,
    history: &[ChatMessage],
) -> Result<AIQueryResponse, String> {
    let client = reqwest::Client::new();
    
    let mut messages = vec![];
    
    for msg in history {
        messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
    }
    
    messages.push(serde_json::json!({"role": "user", "content": query}));

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 4096,
            "system": context,
            "messages": messages
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Anthropic API error {}: {}", status, error_text));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = data["content"][0]["text"]
        .as_str()
        .ok_or("No content in response")?
        .to_string();

    let tokens = data["usage"]["input_tokens"]
        .as_u64()
        .and_then(|i| data["usage"]["output_tokens"].as_u64().map(|o| (i + o) as u32));

    Ok(AIQueryResponse {
        response: content,
        model_used: "claude-sonnet-4".to_string(),
        tokens_used: tokens,
    })
}

/// Call Gemini API
async fn call_gemini_api(
    api_key: &str,
    query: &str,
    context: &str,
) -> Result<AIQueryResponse, String> {
    let client = reqwest::Client::new();
    let prompt = format!("{}\n\nUser Query: {}", context, query);

    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        ))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Gemini API error {}: {}", status, error_text));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = data["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("No content in response")?
        .to_string();

    let tokens = data["usageMetadata"]["totalTokenCount"].as_u64().map(|t| t as u32);

    Ok(AIQueryResponse {
        response: content,
        model_used: "gemini-2.5-flash".to_string(),
        tokens_used: tokens,
    })
}