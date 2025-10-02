
## Why I Built This

Traditional network security tools are either expensive enterprise products or basic open-source utilities that lack modern ML capabilities. I wanted something in between - a production-quality IDS that's free to run, demonstrates real ML engineering skills, and actually works.

Built this to learn:
- Low-level network programming and packet analysis
- Neural network inference in Rust (not Python)
- Real-time data pipelines with async Rust
- Practical ML ops (training, deployment, monitoring)
- Full-stack security tooling

The AI chat feature came from frustration with parsing security logs manually. Instead of reading through hundreds of alerts, you can just ask "what's happening?" in plain English and get actionable answers.

Chose Rust for the backend because network security tools need to be fast and memory-safe - can't have a security tool that crashes or has buffer overflows. The simulation mode exists because developing with real packet capture is a pain (needs root, different setup per OS, can't reliably test attack scenarios).

This is the kind of tool I'd actually use, not just a resume project.

```markdown
# Network IDS

ML-powered network intrusion detection with web dashboard, terminal UI, and AI chat.

## What it does

- Captures network packets (real via libpcap or simulated for testing)
- Extracts statistical features from packet flows
- Runs neural network to score anomaly likelihood
- Detects known attack patterns (port scans, SYN floods, DDoS)
- Real-time alerts via WebSocket
- Natural language queries with AI (GPT-4o, Claude, Gemini)

## Interfaces

**Web Dashboard** - Real-time charts, alert feed, IP geolocation
**Terminal UI** - Interactive CLI with live stats and threat visualization
**AI Chat** - Ask security questions in plain English

## Stack

**Backend:**
- Rust (Tokio async, Axum REST API)
- Candle ML for neural network inference
- pcap for packet capture
- reqwest for AI API calls

**Frontend:**
- SvelteKit + TypeScript
- Chart.js for visualizations
- WebSocket for live updates

**AI Integration:**
- OpenAI GPT-4o
- Anthropic Claude Sonnet 4
- Google Gemini 2.5 Flash

## Setup

**Prerequisites:**
- Rust 1.75+
- Node.js 18+ with pnpm
- Npcap (Windows) or libpcap (Linux/Mac) for real capture

**Install:**
```bash
# Backend
cd network-ids-api
cargo build --release

# Frontend  
cd web
pnpm install
pnpm dev

# CLI
cargo build --release --bin network-ids
```

**Run web interface:**
```bash
# Terminal 1: API server
cargo run --bin network-ids-api

# Terminal 2: Web UI
cd web && pnpm dev
```

API at `http://localhost:3000`
Dashboard at `http://localhost:5173`

**Run CLI:**
```bash
cargo run --bin network-ids start --interface eth0
```

## Configuration

Create `.env` in project root:
```env
ABUSEIPDB_API_KEY=your_key_here
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...
```

Set network interface in config:
```rust
interface: "Wi-Fi".to_string(),  // Windows
// interface: "eth0".to_string(),  // Linux
// interface: "en0".to_string(),   // macOS
```

## Real vs Simulated Packets

**Real mode** (requires admin/sudo):
```bash
# Windows
Right-click terminal → Run as Administrator

# Linux/Mac
sudo cargo run --bin network-ids-api
```

**Simulated mode** (default on Windows):
Generates synthetic traffic for testing. No special privileges needed.

## Architecture

```
Packet Capture
    ↓
Feature Extraction (flow statistics)
    ↓
ML Model (anomaly scoring)
    ↓
Detection Engine (rule matching)
    ↓
Alert System
    ↓
├─→ WebSocket → Web Dashboard
├─→ Terminal UI
└─→ AI Chat Context
```

## ML Model

Neural network trained on flow-level features:
- Packet/byte counts, flow duration
- Protocol distribution, port entropy
- TCP flag patterns, packet size variance
- Inter-arrival timing statistics

Outputs anomaly score 0-1. Threshold 0.7 triggers alert.

## Detection Methods

**ML-based:** Anomaly detection via neural network
**Rule-based:** Pattern matching for known attacks
**Hybrid:** Combines both for high accuracy with low false positives

## Alert Types

- Port scans (horizontal/vertical)
- SYN floods, DDoS attacks
- Unusual protocols, suspicious flag combinations
- High entropy in packet distributions
- Anomalous source/destination patterns

Each alert includes:
- Anomaly score + confidence
- Attack classification
- Source/dest IPs and ports
- IP geolocation + reputation (AbuseIPDB)
- Recommended actions

## AI Assistant

Ask security questions in natural language:
- "What are the top 3 threats?"
- "Analyze the recent SYN flood alert"
- "Show me traffic patterns from 192.168.1.x"
- "Generate a security report"

Supports switching between OpenAI, Anthropic, and Google models.

## CLI Commands

```bash
# Start monitoring
network-ids start --interface eth0 --sensitivity 0.7

# Use simulation mode
network-ids start --simulate

# Show config
network-ids config

# Check status  
network-ids status
```

## Performance

- Processes ~1000 packets/second on typical hardware
- Memory: ~50MB baseline, ~200MB under load
- Sub-second alert latency
- Web UI updates at 1Hz via WebSocket

## Known Issues

- Windows requires Npcap for real capture
- ML model needs custom training for specific networks
- Initial false positive rate higher (model adapts over time)
- Terminal UI may have rendering issues on some terminals

## Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build for release
cargo build --release
```

## License

MIT

## Contributing

PRs welcome. Focus areas:
- Additional ML models (RNN, Transformer)
- More attack signatures
- Performance optimizations
- Better visualization
```
