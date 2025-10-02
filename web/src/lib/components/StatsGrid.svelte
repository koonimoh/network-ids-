<script lang="ts">
	import type { SystemStats } from '$lib/types';

	export let stats: SystemStats;

	function formatBytes(bytes: number): string {
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let size = bytes;
		let unitIndex = 0;
		while (size >= 1024 && unitIndex < units.length - 1) {
			size /= 1024;
			unitIndex++;
		}
		return `${size.toFixed(2)} ${units[unitIndex]}`;
	}

	function formatNumber(num: number): string {
		return new Intl.NumberFormat().format(num);
	}
</script>

<div class="stats-grid">
	<div class="stat-card">
		<div class="stat-icon" style="--color: var(--accent-primary)">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
			</svg>
		</div>
		<div class="stat-content">
			<span class="stat-label">Packets Processed</span>
			<span class="stat-value mono">{formatNumber(stats.packets_processed)}</span>
		</div>
	</div>

	<div class="stat-card">
		<div class="stat-icon" style="--color: var(--info)">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
			</svg>
		</div>
		<div class="stat-content">
			<span class="stat-label">Data Processed</span>
			<span class="stat-value mono">{formatBytes(stats.bytes_processed)}</span>
		</div>
	</div>

	<div class="stat-card">
		<div class="stat-icon" style="--color: var(--error)">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
				<line x1="12" y1="9" x2="12" y2="13"/>
				<line x1="12" y1="17" x2="12.01" y2="17"/>
			</svg>
		</div>
		<div class="stat-content">
			<span class="stat-label">Threats Detected</span>
			<span class="stat-value mono">{formatNumber(stats.threats_detected)}</span>
		</div>
	</div>

	<div class="stat-card">
		<div class="stat-icon" style="--color: var(--success)">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M12 20v-6M6 20V10M18 20V4"/>
			</svg>
		</div>
		<div class="stat-content">
			<span class="stat-label">Processing Rate</span>
			<span class="stat-value mono">{stats.processing_rate.toFixed(2)} <span class="unit">pps</span></span>
		</div>
	</div>

	<div class="stat-card">
		<div class="stat-icon" style="--color: var(--warning)">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="10"/>
				<polyline points="12 6 12 12 16 14"/>
			</svg>
		</div>
		<div class="stat-content">
			<span class="stat-label">Active Flows</span>
			<span class="stat-value mono">{formatNumber(stats.active_flows)}</span>
		</div>
	</div>

	<div class="stat-card">
		<div class="stat-icon" style="--color: var(--accent-primary)">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M18 20V10M12 20V4M6 20v-6"/>
			</svg>
		</div>
		<div class="stat-content">
			<span class="stat-label">CPU Usage</span>
			<span class="stat-value mono">{stats.cpu_usage.toFixed(1)}<span class="unit">%</span></span>
		</div>
	</div>
</div>

<style>
	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
		gap: 1rem;
	}

	.stat-card {
		display: flex;
		align-items: center;
		gap: 1rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.25rem;
		transition: all 0.2s;
	}

	.stat-card:hover {
		border-color: var(--border-secondary);
		box-shadow: var(--shadow-sm);
	}

	.stat-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		border-radius: 0.5rem;
		background: color-mix(in srgb, var(--color) 10%, transparent);
		color: var(--color);
		flex-shrink: 0;
	}

	.stat-content {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		min-width: 0;
	}

	.stat-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.stat-value {
		font-size: 1.5rem;
		font-weight: 600;
		color: var(--text-primary);
		line-height: 1;
	}

	.unit {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-secondary);
	}

	@media (max-width: 768px) {
		.stats-grid {
			grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		}

		.stat-value {
			font-size: 1.25rem;
		}
	}
</style>