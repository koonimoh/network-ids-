export interface SystemStats {
	start_time: string;
	packets_processed: number;
	bytes_processed: number;
	threats_detected: number;
	processing_rate: number;
	memory_usage: number;
	cpu_usage: number;
	active_flows: number;
	alert_counts: Record<string, number>;
	protocol_distribution: Record<string, number>;
	top_talkers: [string, number][];
}

export interface ThreatAlert {
	id: string;
	timestamp: string;
	severity: 'Low' | 'Medium' | 'High' | 'Critical';
	threat_type: string;
	confidence: number;
	anomaly_score: number;
	source_ip: string;
	target_ip: string | null;
	affected_ports: number[];
	description: string;
	explanation: ThreatExplanation;
	raw_packets: string[];
}

export interface ThreatExplanation {
	primary_indicators: string[];
	feature_importance: Record<string, number>;
	similar_incidents: string[];
	recommended_actions: string[];
}

export interface SystemStatus {
	running: boolean;
	uptime_seconds: number;
	version: string;
}

export interface SystemConfig {
	interface: string;
	sensitivity: number;
	max_pps: number;
	ml_config: MLConfig;
	alert_thresholds: AlertThresholds;
	use_simulation: boolean;
}

export interface MLConfig {
	update_frequency: number;
	batch_size: number;
	learning_rate: number;
	window_size: number;
}

export interface AlertThresholds {
	anomaly_threshold: number;
	min_confidence: number;
	max_alerts_per_minute: number;
}

export interface ApiResponse<T> {
	success: boolean;
	data: T | null;
	error: string | null;
	timestamp: string;
}

export interface ExportOptions {
	format: 'json' | 'csv';
	includeAlerts: boolean;
	includeStats: boolean;
	timeRange?: {
		start: string;
		end: string;
	};
	
	// Alert Acknowledgment System types
	export type AlertStatus = 'new' | 'reviewed' | 'investigating' | 'resolved' | 'false_positive';

	export interface AlertAcknowledgment {
		alertId: string;
		status: AlertStatus;
		acknowledgedAt: string;
		notes?: string;
}


export interface ThreatLocation {
	ip: string;
	latitude: number;
	longitude: number;
	country: string;
	city: string | null;
	count: number;
	severity: 'Low' | 'Medium' | 'High' | 'Critical';
}

export interface GeolocationResponse {
	success: boolean;
	data: ThreatLocation[];
}