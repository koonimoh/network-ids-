import type { AIProvider } from '$lib/stores/ai';
import type { SystemStats, ThreatAlert } from '$lib/types';

interface Message {
	role: 'user' | 'assistant' | 'system';
	content: string;
}

export class AIService {
	private apiKey: string;
	private provider: AIProvider;
	private model: string;

	constructor(apiKey: string, provider: AIProvider, model: string) {
		this.apiKey = apiKey;
		this.provider = provider;
		this.model = model;
	}

	async sendMessage(
		userMessage: string,
		systemContext: string,
		conversationHistory: Message[]
	): Promise<string> {
		if (this.provider === 'openai') {
			return this.sendOpenAIMessage(userMessage, systemContext, conversationHistory);
		} else {
			return this.sendAnthropicMessage(userMessage, systemContext, conversationHistory);
		}
	}

	private async sendOpenAIMessage(
		userMessage: string,
		systemContext: string,
		conversationHistory: Message[]
	): Promise<string> {
		const messages: Message[] = [
			{ role: 'system', content: systemContext },
			...conversationHistory,
			{ role: 'user', content: userMessage }
		];

		const response = await fetch('https://api.openai.com/v1/chat/completions', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				'Authorization': `Bearer ${this.apiKey}`
			},
			body: JSON.stringify({
				model: this.model,
				messages: messages,
				temperature: 0.7,
				max_tokens: 2000
			})
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error?.message || 'OpenAI API request failed');
		}

		const data = await response.json();
		return data.choices[0].message.content;
	}

	private async sendAnthropicMessage(
		userMessage: string,
		systemContext: string,
		conversationHistory: Message[]
	): Promise<string> {
		// Filter out system messages from history for Anthropic
		const filteredHistory = conversationHistory.filter(msg => msg.role !== 'system');

		const response = await fetch('https://api.anthropic.com/v1/messages', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				'x-api-key': this.apiKey,
				'anthropic-version': '2023-06-01'
			},
			body: JSON.stringify({
				model: this.model,
				max_tokens: 4096,
				system: systemContext,
				messages: [
					...filteredHistory,
					{ role: 'user', content: userMessage }
				]
			})
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error?.message || 'Anthropic API request failed');
		}

		const data = await response.json();
		return data.content[0].text;
	}

	static buildContext(stats: SystemStats | null, alerts: ThreatAlert[]): string {
		let context = 'You are an expert cybersecurity analyst helping to analyze network intrusion detection system (IDS) data. ';
		context += 'Provide clear, actionable insights about security threats, patterns, and recommendations.\n\n';

		if (stats) {
			context += '## System Statistics\n';
			context += `- Total packets processed: ${stats.packets_processed.toLocaleString()}\n`;
			context += `- Total bytes processed: ${stats.bytes_processed.toLocaleString()}\n`;
			context += `- Threats detected: ${stats.threats_detected.toLocaleString()}\n`;
			context += `- Processing rate: ${stats.processing_rate.toFixed(2)} packets/second\n`;
			context += `- Active network flows: ${stats.active_flows}\n`;
			context += `- CPU usage: ${stats.cpu_usage.toFixed(1)}%\n`;
			context += `- Memory usage: ${(stats.memory_usage / 1024 / 1024).toFixed(2)} MB\n\n`;

			if (Object.keys(stats.protocol_distribution).length > 0) {
				context += '## Protocol Distribution\n';
				Object.entries(stats.protocol_distribution).forEach(([protocol, count]) => {
					context += `- ${protocol}: ${count.toLocaleString()} packets\n`;
				});
				context += '\n';
			}

			if (Object.keys(stats.alert_counts).length > 0) {
				context += '## Alert Distribution by Severity\n';
				Object.entries(stats.alert_counts).forEach(([severity, count]) => {
					context += `- ${severity}: ${count} alerts\n`;
				});
				context += '\n';
			}

			if (stats.top_talkers.length > 0) {
				context += '## Top Network Talkers\n';
				stats.top_talkers.slice(0, 5).forEach(([ip, bytes], index) => {
					context += `${index + 1}. ${ip}: ${(bytes / 1024 / 1024).toFixed(2)} MB\n`;
				});
				context += '\n';
			}
		}

		if (alerts.length > 0) {
			context += `## Recent Threat Alerts (${alerts.length} total)\n`;
			const recentAlerts = alerts.slice(0, 10);
			
			recentAlerts.forEach((alert, index) => {
				context += `\n### Alert ${index + 1}\n`;
				context += `- Type: ${alert.threat_type}\n`;
				context += `- Severity: ${alert.severity}\n`;
				context += `- Source IP: ${alert.source_ip}\n`;
				if (alert.target_ip) context += `- Target IP: ${alert.target_ip}\n`;
				context += `- Confidence: ${(alert.confidence * 100).toFixed(0)}%\n`;
				context += `- Description: ${alert.description}\n`;
				
				if (alert.explanation.primary_indicators.length > 0) {
					context += `- Indicators: ${alert.explanation.primary_indicators.join(', ')}\n`;
				}
			});
		}

		return context;
	}
}