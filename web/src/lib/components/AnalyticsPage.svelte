<script lang="ts">
	import { wsStore } from '$lib/stores/websocket';
	import type { SystemStats, ThreatAlert, ApiResponse } from '$lib/types';

	export let stats: SystemStats | null;

	let exporting = false;
	let exportFormat: 'json' | 'csv' = 'json';
	let includeAlerts = true;
	let includeStats = true;

	async function exportData() {
		if (!includeAlerts && !includeStats) {
			alert('Please select at least one data type to export');
			return;
		}

		exporting = true;
		try {
			const exportData: any = {
				timestamp: new Date().toISOString(),
				exported_at: new Date().toLocaleString()
			};

			if (includeStats && stats) {
				exportData.statistics = {
					...stats,
					uptime_seconds: Math.floor((new Date().getTime() - new Date(stats.start_time).getTime()) / 1000)
				};
			}

			if (includeAlerts) {
				// Fetch all alerts from backend
				const res = await fetch('/api/alerts?limit=1000');
				const data: ApiResponse<ThreatAlert[]> = await res.json();
				if (data.success && data.data) {
					exportData.alerts = data.data;
				}
			}

			if (exportFormat === 'json') {
				const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
				downloadFile(blob, `ids-export-${Date.now()}.json`);
			} else {
				const csv = convertToCSV(exportData);
				const blob = new Blob([csv], { type: 'text/csv' });
				downloadFile(blob, `ids-export-${Date.now()}.csv`);
			}
		} catch (err) {
			console.error('Export failed:', err);
			alert('Failed to export data');
		} finally {
			exporting = false;
		}
	}

	function convertToCSV(data: any): string {
		let csv = '';

		// Stats section
		if (data.statistics) {
			csv += 'STATISTICS\n';
			csv += 'Metric,Value\n';
			csv += `Packets Processed,${data.statistics.packets_processed}\n`;
			csv += `Bytes Processed,${data.statistics.bytes_processed}\n`;
			csv += `Threats Detected,${data.statistics.threats_detected}\n`;
			csv += `Processing Rate,${data.statistics.processing_rate}\n`;
			csv += `Active Flows,${data.statistics.active_flows}\n`;
			csv += `CPU Usage,${data.statistics.cpu_usage}\n`;
			csv += `Memory Usage,${data.statistics.memory_usage}\n`;
			csv += '\n';
		}

		// Alerts section
		if (data.alerts && data.alerts.length > 0) {
			csv += 'ALERTS\n';
			csv += 'Timestamp,Severity,Type,Source IP,Target IP,Confidence,Description\n';
			data.alerts.forEach((alert: ThreatAlert) => {
				csv += `${alert.timestamp},${alert.severity},${alert.threat_type},${alert.source_ip},${alert.target_ip || 'N/A'},${alert.confidence},${alert.description.replace(/,/g, ';')}\n`;
			});
		}

		return csv;
	}

	function downloadFile(blob: Blob, filename: string) {
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}

	function calculateThreatTrend(): number {
		if (!stats) return 0;
		const total = Object.values(stats.alert_counts).reduce((sum, count) => sum + count, 0);
		return total > 0 ? ((stats.threats_detected / total) * 100) : 0;
	}

	$: totalAlerts = stats ? Object.values(stats.alert_counts).reduce((sum, count) => sum + count, 0) : 0;
</script>

<div class="analytics-page">
	<div class="page-header">
		<h1>Analytics & Reports</h1>
		<p class="subtitle">Export and analyze system data</p>
	</div>

	<div class="analytics-grid">
		<div class="card summary-card">
			<h3>System Summary</h3>
			{#if stats}
				<div class="summary-stats">
					<div class="summary-item">
						<span class="summary-label">Total Packets</span>
						<span class="summary-value mono">{stats.packets_processed.toLocaleString()}</span>
					</div>
					<div class="summary-item">
						<span class="summary-label">Total Threats</span>
						<span class="summary-value mono">{stats.threats_detected.toLocaleString()}</span>
					</div>
					<div class="summary-item">
						<span class="summary-label">Alert Distribution</span>
						<span class="summary-value mono">{totalAlerts} total</span>
					</div>
					<div class="summary-item">
						<span class="summary-label">Threat Rate</span>
						<span class="summary-value mono">{calculateThreatTrend().toFixed(2)}%</span>
					</div>
				</div>
			{:else}
				<p class="empty-text">No data available. Start the IDS to collect statistics.</p>
			{/if}
		</div>

		<div class="card export-card">
			<h3>Export Data</h3>
			<div class="export-options">
				<div class="option-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={includeStats} />
						<span>Include Statistics</span>
					</label>
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={includeAlerts} />
						<span>Include Alerts</span>
					</label>
				</div>

				<div class="format-selector">
					<label class="format-label">Export Format</label>
					<div class="format-buttons">
						<button 
							class="format-btn"
							class:active={exportFormat === 'json'}
							on:click={() => exportFormat = 'json'}
						>
							JSON
						</button>
						<button 
							class="format-btn"
							class:active={exportFormat === 'csv'}
							on:click={() => exportFormat = 'csv'}
						>
							CSV
						</button>
					</div>
				</div>

				<button 
					class="btn-primary"
					on:click={exportData}
					disabled={exporting || (!includeStats && !includeAlerts)}
				>
					{#if exporting}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
							<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
						</svg>
						Exporting...
					{:else}
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
							<polyline points="7 10 12 15 17 10"/>
							<line x1="12" y1="15" x2="12" y2="3"/>
						</svg>
						Export Data
					{/if}
				</button>
			</div>
		</div>
	</div>

	<div class="card alerts-summary">
		<div class="card-header">
			<h3>Recent Alert History</h3>
			<span class="alert-count">{$wsStore.alerts.length} alerts</span>
		</div>
		<div class="alert-timeline">
			{#if $wsStore.alerts.length > 0}
				{#each $wsStore.alerts.slice(0, 20) as alert}
					<div class="timeline-item">
						<div class="timeline-dot" style="background: var(--severity-{alert.severity.toLowerCase()})"></div>
						<div class="timeline-content">
							<div class="timeline-header">
								<span class="timeline-type">{alert.threat_type}</span>
								<span class="timeline-time mono">{new Date(alert.timestamp).toLocaleString()}</span>
							</div>
							<p class="timeline-desc">{alert.description}</p>
						</div>
					</div>
				{/each}
			{:else}
				<p class="empty-text">No alerts recorded yet</p>
			{/if}
		</div>
	</div>
</div>

<style>
	.analytics-page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.page-header h1 {
		font-size: 1.875rem;
		font-weight: 700;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		color: var(--text-secondary);
		font-size: 0.875rem;
	}

	.analytics-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.card h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 1.25rem;
	}

	.summary-stats {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1rem;
	}

	.summary-item {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		padding: 1rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
	}

	.summary-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.summary-value {
		font-size: 1.25rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.export-options {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.option-group {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		cursor: pointer;
		font-size: 0.875rem;
		color: var(--text-primary);
	}

	.checkbox-label input[type="checkbox"] {
		width: 1.125rem;
		height: 1.125rem;
		cursor: pointer;
	}

	.format-selector {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.format-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-secondary);
	}

	.format-buttons {
		display: flex;
		gap: 0.5rem;
	}

	.format-btn {
		flex: 1;
		padding: 0.625rem;
		border-radius: 0.5rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-secondary);
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.format-btn:hover {
		background: var(--bg-hover);
		border-color: var(--border-secondary);
	}

	.format-btn.active {
		background: var(--accent-primary);
		border-color: var(--accent-primary);
		color: var(--text-inverse);
	}

	.btn-primary {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.75rem 1.5rem;
		border-radius: 0.5rem;
		background: var(--accent-primary);
		color: var(--text-inverse);
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.alerts-summary {
		grid-column: 1 / -1;
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1.25rem;
	}

	.card-header h3 {
		margin: 0;
	}

	.alert-count {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-secondary);
		padding: 0.25rem 0.625rem;
		background: var(--bg-tertiary);
		border-radius: 0.25rem;
	}

	.alert-timeline {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		max-height: 500px;
		overflow-y: auto;
	}

	.timeline-item {
		display: flex;
		gap: 1rem;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
	}

	.timeline-dot {
		width: 0.75rem;
		height: 0.75rem;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 0.25rem;
	}

	.timeline-content {
		flex: 1;
		min-width: 0;
	}

	.timeline-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 0.25rem;
		gap: 1rem;
	}

	.timeline-type {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.timeline-time {
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	.timeline-desc {
		font-size: 0.875rem;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.empty-text {
		padding: 2rem;
		text-align: center;
		color: var(--text-secondary);
		font-size: 0.875rem;
	}

	@media (max-width: 768px) {
		.analytics-grid {
			grid-template-columns: 1fr;
		}

		.summary-stats {
			grid-template-columns: 1fr;
		}
	}
</style>