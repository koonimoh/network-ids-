<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { ThreatLocation } from '$lib/types';
	import { Deck } from '@deck.gl/core';
	import { ScatterplotLayer } from '@deck.gl/layers';
	import { HeatmapLayer } from '@deck.gl/aggregation-layers';

	let mapContainer: HTMLDivElement;
	let deck: any;
	let map: any;
	let loading = true;
	let error: string | null = null;
	let locations: ThreatLocation[] = [];
	let refreshInterval: ReturnType<typeof setInterval>;
	let showHeatmap = true;
	let animationFrame: number;

	const severityColors: Record<string, [number, number, number]> = {
		Low: [59, 130, 246],
		Medium: [245, 158, 11],
		High: [239, 68, 68],
		Critical: [220, 38, 38]
	};

	async function fetchThreatLocations() {
		try {
			const response = await fetch('/api/geolocation');
			const data = await response.json();
			
			if (data.success && data.data) {
				locations = data.data;
				updateLayers();
			}
		} catch (err) {
			console.error('Failed to fetch threat locations:', err);
			error = 'Failed to load threat data';
		}
	}

	function updateLayers() {
		if (!deck || locations.length === 0) return;

		const scatterLayer = new ScatterplotLayer({
			id: 'threats-scatter',
			data: locations,
			getPosition: (d: ThreatLocation) => [d.longitude, d.latitude],
			getRadius: (d: ThreatLocation) => Math.min(d.count * 50000, 500000),
			getColor: (d: ThreatLocation) => {
				const color = severityColors[d.severity] || [59, 130, 246];
				return [...color, 180];
			},
			pickable: true,
			radiusMinPixels: 10,
			radiusMaxPixels: 100,
			transitions: {
				getRadius: 500
			}
		});

		const heatmapLayer = new HeatmapLayer({
			id: 'threats-heatmap',
			data: locations,
			getPosition: (d: ThreatLocation) => [d.longitude, d.latitude],
			getWeight: (d: ThreatLocation) => d.count,
			radiusPixels: 60,
			intensity: 1,
			threshold: 0.05,
			visible: showHeatmap,
			aggregation: 'SUM'
		});

		deck.setProps({
			layers: [heatmapLayer, scatterLayer]
		});
	}

	async function initializeDeck() {
		const maplibregl = await import('maplibre-gl');
		
		map = new maplibregl.Map({
			container: mapContainer,
			style: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
			center: [0, 20],
			zoom: 1.5,
			pitch: 0,
			interactive: false
		});

		await new Promise(resolve => map.on('load', resolve));

		deck = new Deck({
			canvas: 'deck-canvas',
			width: '100%',
			height: '100%',
			initialViewState: {
				longitude: 0,
				latitude: 20,
				zoom: 1.5,
				pitch: 0,
				bearing: 0
			},
			controller: true,
			onViewStateChange: ({viewState}: any) => {
				map.jumpTo({
					center: [viewState.longitude, viewState.latitude],
					zoom: viewState.zoom,
					bearing: viewState.bearing,
					pitch: viewState.pitch
				});
			},
			getTooltip: ({object}: any) => {
				if (!object) return null;
				const loc = object as ThreatLocation;
				return {
					html: `
						<div style="font-family: system-ui; padding: 12px; background: #1e293b; border-radius: 8px; color: #e2e8f0;">
							<div style="font-weight: 600; font-size: 14px; margin-bottom: 8px; color: rgb(${severityColors[loc.severity].join(',')});">
								${loc.severity} Threat
							</div>
							<div style="font-size: 13px; margin-bottom: 4px;">
								<strong>Location:</strong> ${loc.city ? loc.city + ', ' : ''}${loc.country}
							</div>
							<div style="font-size: 13px; margin-bottom: 4px;">
								<strong>Source IP:</strong> <code style="background: #0f172a; padding: 2px 6px; border-radius: 3px;">${loc.ip}</code>
							</div>
							<div style="font-size: 13px;">
								<strong>Incidents:</strong> ${loc.count}
							</div>
						</div>
					`,
					style: {
						backgroundColor: 'transparent',
						fontSize: '0.8em'
					}
				};
			}
		});

		loading = false;
	}

	function animate() {
		if (deck && locations.length > 0) {
			updateLayers();
		}
		animationFrame = requestAnimationFrame(animate);
	}

	onMount(async () => {
		if (typeof window !== 'undefined') {
			await fetchThreatLocations();
			await initializeDeck();
			animate();
			refreshInterval = setInterval(fetchThreatLocations, 30000);
		}
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
		if (animationFrame) cancelAnimationFrame(animationFrame);
		if (deck) deck.finalize();
		if (map) map.remove();
	});
</script>

<svelte:head>
	<link rel="stylesheet" href="https://unpkg.com/maplibre-gl@4.0.0/dist/maplibre-gl.css" />
</svelte:head>

<div class="threat-map-container">
	<div class="map-header">
		<div>
			<h3>Global Threat Map</h3>
			<p class="subtitle">Real-time visualization with animated threat indicators</p>
		</div>
		<div class="map-controls">
			<div class="map-stats">
				<div class="stat-item">
					<span class="stat-label">Active Threats</span>
					<span class="stat-value">{locations.length}</span>
				</div>
				<div class="stat-item">
					<span class="stat-label">Total Incidents</span>
					<span class="stat-value">{locations.reduce((sum, loc) => sum + loc.count, 0)}</span>
				</div>
			</div>
			<button 
				class="toggle-btn"
				class:active={showHeatmap}
				on:click={() => { showHeatmap = !showHeatmap; updateLayers(); }}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M12 2L2 7l10 5 10-5-10-5z"/>
					<path d="M2 17l10 5 10-5M2 12l10 5 10-5"/>
				</svg>
				Heatmap
			</button>
		</div>
	</div>

	<div class="map-legend">
		<span class="legend-title">Severity:</span>
		{#each Object.entries(severityColors) as [severity, color]}
			<div class="legend-item">
				<div class="legend-dot" style="background: rgb({color.join(',')})"></div>
				<span>{severity}</span>
			</div>
		{/each}
	</div>

	{#if loading}
		<div class="map-loading">
			<svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
				<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
			</svg>
			<p>Loading threat map...</p>
		</div>
	{:else if error}
		<div class="map-error">
			<svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="10"/>
				<line x1="12" y1="8" x2="12" y2="12"/>
				<line x1="12" y1="16" x2="12.01" y2="16"/>
			</svg>
			<p>{error}</p>
		</div>
	{/if}

	<div class="map-wrapper" class:hidden={loading || error}>
		<div bind:this={mapContainer} class="map"></div>
		<canvas id="deck-canvas"></canvas>
	</div>
</div>

<style>
	.threat-map-container {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.map-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.5rem;
		border-bottom: 1px solid var(--border-primary);
		flex-wrap: wrap;
		gap: 1rem;
	}

	h3 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0 0 0.25rem 0;
	}

	.subtitle {
		font-size: 0.875rem;
		color: var(--text-secondary);
		margin: 0;
	}

	.map-controls {
		display: flex;
		align-items: center;
		gap: 1.5rem;
	}

	.map-stats {
		display: flex;
		gap: 2rem;
	}

	.stat-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.stat-label {
		font-size: 0.75rem;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.stat-value {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--accent-primary);
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.toggle-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.toggle-btn.active {
		background: var(--accent-primary);
		border-color: var(--accent-primary);
		color: var(--text-inverse);
	}

	.map-legend {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem 1.5rem;
		background: var(--bg-tertiary);
		border-bottom: 1px solid var(--border-primary);
		flex-wrap: wrap;
	}

	.legend-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-secondary);
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: var(--text-secondary);
	}

	.legend-dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		box-shadow: 0 0 8px currentColor;
	}

	.map-wrapper {
		position: relative;
		height: 600px;
		background: #0a0e1a;
	}

	.map-wrapper.hidden {
		display: none;
	}

	.map {
		width: 100%;
		height: 100%;
		position: absolute;
		top: 0;
		left: 0;
	}

	#deck-canvas {
		position: absolute;
		left: 0;
		top: 0;
		width: 100%;
		height: 100%;
		pointer-events: auto;
	}

	.map-loading,
	.map-error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 600px;
		gap: 1rem;
		color: var(--text-secondary);
		background: #0a0e1a;
	}

	.map-error {
		color: var(--error);
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	@media (max-width: 768px) {
		.map-header {
			flex-direction: column;
			align-items: flex-start;
		}

		.map-controls {
			width: 100%;
			flex-direction: column;
			align-items: flex-start;
		}

		.map-stats {
			width: 100%;
			justify-content: space-between;
		}

		.map-wrapper {
			height: 400px;
		}
	}
</style>