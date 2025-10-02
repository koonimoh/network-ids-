<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { wsStore } from '$lib/stores/websocket';
	import { historyStore } from '$lib/stores/history'; 
	import { currentPage } from '$lib/stores/navigation';
	import type { SystemStats, SystemStatus, ApiResponse } from '$lib/types';
	
	import StatusCard from '$lib/components/StatusCard.svelte';
	import StatsGrid from '$lib/components/StatsGrid.svelte';
	import AlertFeed from '$lib/components/AlertFeed.svelte';
	import ProtocolChart from '$lib/components/ProtocolChart.svelte';
	import ThreatChart from '$lib/components/ThreatChart.svelte';
	import TopTalkers from '$lib/components/TopTalkers.svelte';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte'; 
	import BlocklistManager from '$lib/components/BlocklistManager.svelte';
	import ConnectionDetailsTable from '$lib/components/ConnectionDetailsTable.svelte'; 
	import AnalyticsPage from '$lib/components/AnalyticsPage.svelte';
	import AISecurityCenter from '$lib/components/AISecurityCenter.svelte';
	import SettingsPage from '$lib/components/SettingsPage.svelte';
	import ThreatMapDeckGL from '$lib/components/ThreatMapDeckGL.svelte';

	let status: SystemStatus | null = null;
	let stats: SystemStats | null = null;
	let loading = false;
	let refreshing = false;
	let statusInterval: ReturnType<typeof setInterval>;
	let statsInterval: ReturnType<typeof setInterval>;

	async function fetchStatus() {
		try {
			const res = await fetch('/api/status');
			const data: ApiResponse<SystemStatus> = await res.json();
			if (data.success && data.data) {
				status = data.data;
			}
		} catch (err) {
			console.error('Failed to fetch status:', err);
		}
	}

	async function fetchStats() {
		try {
			const res = await fetch('/api/stats');
			const data: ApiResponse<SystemStats> = await res.json();
			if (data.success && data.data) {
				stats = data.data;
				// Add to history store for time-series chart
				historyStore.addDataPoint(data.data);
			}
		} catch (err) {
			console.error('Failed to fetch stats:', err);
		}
	}

	async function refreshData() {
		refreshing = true;
		try {
			await Promise.all([fetchStatus(), fetchStats()]);
		} finally {
			refreshing = false;
		}
	}

	async function startIDS() {
		console.log('⭐ startIDS() called');
		loading = true;
		try {
			console.log('📡 Fetching /api/start');
			const res = await fetch('/api/start', { method: 'POST' });
			console.log('📡 Response status:', res.status);
			
			const data: ApiResponse<string> = await res.json();
			console.log('📡 Response data:', data);
			
			if (data.success) {
				console.log('✅ API success, calling wsStore.connect()');
				wsStore.connect();
				await fetchStatus();
			}else {
				console.log('❌ API returned success=false');
			}
		} catch (err) {
			console.error('❌ Error in startIDS:', err);
		} finally {
			loading = false;
		}
	}

	async function stopIDS() {
		loading = true;
		try {
			const res = await fetch('/api/stop', { method: 'POST' });
			const data: ApiResponse<string> = await res.json();
			if (data.success) {
				wsStore.disconnect();
				wsStore.clearAlerts();
				historyStore.clear();  // <-- ADD THIS LINE
				await fetchStatus();
			}
		} catch (err) {
			console.error('Failed to stop IDS:', err);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		fetchStatus();
		fetchStats();
		
		statusInterval = setInterval(fetchStatus, 2000);
		statsInterval = setInterval(fetchStats, 1000);
	});

	onDestroy(() => {
		if (statusInterval) clearInterval(statusInterval);
		if (statsInterval) clearInterval(statsInterval);
		wsStore.disconnect();
	});
</script>

<svelte:head>
	<title>Network IDS Dashboard</title>
</svelte:head>

{#if $currentPage === 'dashboard'}
	<div class="dashboard">
		<div class="dashboard-header">
			<div>
				<h1>Intrusion Detection System</h1>
				<p class="subtitle">Real-time network threat monitoring and analysis</p>
			</div>
			
			<button 
				class="btn-icon" 
				on:click={refreshData}
				disabled={refreshing}
				title="Refresh data"
			>
				<svg 
					width="20" 
					height="20" 
					viewBox="0 0 24 24" 
					fill="none" 
					stroke="currentColor" 
					stroke-width="2"
					class:spinning={refreshing}
				>
					<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
				</svg>
			</button>
		</div>

		<StatusCard 
			{status} 
			{loading} 
			on:start={startIDS}
			on:stop={stopIDS}
		/>

		{#if stats}
			<StatsGrid {stats} />
			<TimeSeriesChart />
			
			<div class="charts-grid">
				<ProtocolChart {stats} />
				<ThreatChart {stats} />
			</div>

			<TopTalkers {stats} />
		{/if}
		
		<ConnectionDetailsTable />
		
		<BlocklistManager />

		<AlertFeed />
	</div>
{:else if $currentPage === 'analytics'}
	<AnalyticsPage {stats} />
{:else if $currentPage === 'ai-assistant'}
	<AISecurityCenter />
{:else if $currentPage === 'settings'}
	<SettingsPage />
{:else if $currentPage === 'threat-map'}
	<div class="threat-map-page">
		<div class="page-header">
			<h1>Global Threat Map</h1>
			<p class="subtitle">Visualize threat origins and patterns across the globe</p>
		</div>
		<ThreatMapDeckGL />
	</div>
{/if}

<style>
	.dashboard {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.dashboard-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 0.5rem;
	}

	h1 {
		font-size: 1.875rem;
		font-weight: 700;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		color: var(--text-secondary);
		font-size: 0.875rem;
	}

	.btn-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		border-radius: 0.5rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.btn-icon:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-secondary);
	}

	.btn-icon:disabled {
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

	.charts-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		gap: 1.5rem;
	}

	@media (max-width: 768px) {
		.charts-grid {
			grid-template-columns: 1fr;
		}
		
		h1 {
			font-size: 1.5rem;
		}
	}
</style>