<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	interface ActiveFlow {
		flow_id: string;
		src_ip: string;
		dst_ip: string;
		src_port: number | null;
		dst_port: number | null;
		protocol: string;
		packets: number;
		bytes: number;
		duration: number;
		flags: string[];
	}

	let flows: ActiveFlow[] = [];
	let loading = true;
	let sortBy: 'bytes' | 'packets' | 'duration' = 'bytes';
	let sortOrder: 'asc' | 'desc' = 'desc';
	let refreshInterval: ReturnType<typeof setInterval>;

	async function fetchFlows() {
		try {
			const res = await fetch('/api/flows');
			const data = await res.json();
			if (data.success && data.data) {
				flows = data.data;
			}
		} catch (err) {
			console.error('Failed to fetch flows:', err);
		} finally {
			loading = false;
		}
	}

	function formatBytes(bytes: number): string {
		const units = ['B', 'KB', 'MB', 'GB'];
		let size = bytes;
		let unitIndex = 0;
		while (size >= 1024 && unitIndex < units.length - 1) {
			size /= 1024;
			unitIndex++;
		}
		return `${size.toFixed(2)} ${units[unitIndex]}`;
	}

	function formatDuration(seconds: number): string {
		if (seconds < 60) return `${seconds}s`;
		const minutes = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${minutes}m ${secs}s`;
	}

	function sortFlows(field: typeof sortBy) {
		if (sortBy === field) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = field;
			sortOrder = 'desc';
		}
	}

	$: sortedFlows = [...flows].sort((a, b) => {
		let comparison = 0;
		switch (sortBy) {
			case 'bytes':
				comparison = a.bytes - b.bytes;
				break;
			case 'packets':
				comparison = a.packets - b.packets;
				break;
			case 'duration':
				comparison = a.duration - b.duration;
				break;
		}
		return sortOrder === 'asc' ? comparison : -comparison;
	});

	onMount(() => {
		fetchFlows();
		refreshInterval = setInterval(fetchFlows, 2000); // Refresh every 2 seconds
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});
</script>

<div class="connection-table card">
	<div class="table-header">
		<div>
			<h3>Active Network Flows</h3>
			<p class="subtitle">{flows.length} active connection{flows.length !== 1 ? 's' : ''}</p>
		</div>
		<button class="refresh-btn" on:click={fetchFlows} disabled={loading}>
			<svg 
				width="16" 
				height="16" 
				viewBox="0 0 24 24" 
				fill="none" 
				stroke="currentColor" 
				stroke-width="2"
				class:spinning={loading}
			>
				<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
			</svg>
		</button>
	</div>

	{#if loading && flows.length === 0}
		<div class="loading-state">
			<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
				<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
			</svg>
			<p>Loading flows...</p>
		</div>
	{:else if flows.length === 0}
		<div class="empty-state">
			<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
				<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
			</svg>
			<p>No active flows</p>
			<span>Start the IDS to monitor network connections</span>
		</div>
	{:else}
		<div class="table-container">
			<table>
				<thead>
					<tr>
						<th>Source</th>
						<th>Destination</th>
						<th>Protocol</th>
						<th class="sortable" on:click={() => sortFlows('packets')}>
							<button class="sort-btn">
								Packets
								{#if sortBy === 'packets'}
									<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										{#if sortOrder === 'asc'}
											<polyline points="18 15 12 9 6 15"/>
										{:else}
											<polyline points="6 9 12 15 18 9"/>
										{/if}
									</svg>
								{/if}
							</button>
						</th>
						<th class="sortable" on:click={() => sortFlows('bytes')}>
							<button class="sort-btn">
								Bytes
								{#if sortBy === 'bytes'}
									<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										{#if sortOrder === 'asc'}
											<polyline points="18 15 12 9 6 15"/>
										{:else}
											<polyline points="6 9 12 15 18 9"/>
										{/if}
									</svg>
								{/if}
							</button>
						</th>
						<th class="sortable" on:click={() => sortFlows('duration')}>
							<button class="sort-btn">
								Duration
								{#if sortBy === 'duration'}
									<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										{#if sortOrder === 'asc'}
											<polyline points="18 15 12 9 6 15"/>
										{:else}
											<polyline points="6 9 12 15 18 9"/>
										{/if}
									</svg>
								{/if}
							</button>
						</th>
						<th>Flags</th>
					</tr>
				</thead>
				<tbody>
					{#each sortedFlows as flow (flow.flow_id)}
						<tr>
							<td>
								<div class="endpoint">
									<span class="ip mono">{flow.src_ip}</span>
									{#if flow.src_port}
										<span class="port mono">:{flow.src_port}</span>
									{/if}
								</div>
							</td>
							<td>
								<div class="endpoint">
									<span class="ip mono">{flow.dst_ip}</span>
									{#if flow.dst_port}
										<span class="port mono">:{flow.dst_port}</span>
									{/if}
								</div>
							</td>
							<td>
								<span class="protocol-badge" class:tcp={flow.protocol === 'TCP'} class:udp={flow.protocol === 'UDP'}>
									{flow.protocol}
								</span>
							</td>
							<td class="numeric">{flow.packets.toLocaleString()}</td>
							<td class="numeric">{formatBytes(flow.bytes)}</td>
							<td class="numeric">{formatDuration(flow.duration)}</td>
							<td>
								{#if flow.flags.length > 0}
									<div class="flags">
										{#each flow.flags.slice(0, 3) as flag}
											<span class="flag-badge">{flag}</span>
										{/each}
										{#if flow.flags.length > 3}
											<span class="flag-more">+{flow.flags.length - 3}</span>
										{/if}
									</div>
								{:else}
									<span class="text-muted">—</span>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.table-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1.5rem;
	}

	h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		font-size: 0.875rem;
		color: var(--text-secondary);
	}

	.refresh-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		border-radius: 0.5rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.refresh-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.refresh-btn:disabled {
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

	.loading-state, .empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 3rem 1rem;
		color: var(--text-secondary);
	}

	.loading-state svg, .empty-state svg {
		margin-bottom: 1rem;
		opacity: 0.5;
	}

	.empty-state p {
		font-size: 1rem;
		font-weight: 500;
		margin-bottom: 0.25rem;
		color: var(--text-primary);
	}

	.empty-state span {
		font-size: 0.875rem;
	}

	.table-container {
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
	}

	thead {
		background: var(--bg-tertiary);
		border-bottom: 1px solid var(--border-primary);
	}

	th {
		padding: 0.75rem 1rem;
		text-align: left;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	th.sortable {
		padding: 0;
	}

	.sort-btn {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		width: 100%;
		padding: 0.75rem 1rem;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.8125rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		transition: all 0.2s;
	}

	.sort-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	tbody tr {
		border-bottom: 1px solid var(--border-primary);
		transition: background 0.2s;
	}

	tbody tr:hover {
		background: var(--bg-hover);
	}

	td {
		padding: 0.875rem 1rem;
		font-size: 0.875rem;
	}

	.endpoint {
		display: flex;
		align-items: baseline;
		gap: 0.125rem;
	}

	.ip {
		color: var(--text-primary);
	}

	.port {
		color: var(--text-tertiary);
		font-size: 0.8125rem;
	}

	.protocol-badge {
		display: inline-block;
		padding: 0.25rem 0.625rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
	}

	.protocol-badge.tcp {
		background: color-mix(in srgb, var(--info) 15%, transparent);
		border-color: var(--info);
		color: var(--info);
	}

	.protocol-badge.udp {
		background: color-mix(in srgb, var(--success) 15%, transparent);
		border-color: var(--success);
		color: var(--success);
	}

	.numeric {
		text-align: right;
		font-family: 'JetBrains Mono', monospace;
		color: var(--text-secondary);
	}

	.flags {
		display: flex;
		gap: 0.25rem;
		flex-wrap: wrap;
	}

	.flag-badge {
		padding: 0.125rem 0.375rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.25rem;
		font-size: 0.6875rem;
		font-weight: 600;
		font-family: 'JetBrains Mono', monospace;
	}

	.flag-more {
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	.text-muted {
		color: var(--text-tertiary);
	}

	@media (max-width: 768px) {
		.table-container {
			font-size: 0.8125rem;
		}

		th, td {
			padding: 0.5rem;
		}
	}
</style>