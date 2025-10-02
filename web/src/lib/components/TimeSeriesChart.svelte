<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Chart, LineController, CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend, Filler } from 'chart.js';
	import { historyStore } from '$lib/stores/history';

	Chart.register(LineController, CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend, Filler);

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;
	let selectedMetric: 'threats' | 'packets' | 'bandwidth' = 'threats';
	let timeRange: '1m' | '5m' | '15m' | '30m' | '1h' = '1m';

	const timeRangePoints: Record<typeof timeRange, number> = {
		'1m': 60,
		'5m': 300,
		'15m': 900,
		'30m': 1800,
		'1h': 3600
	};

	function getComputedColor(varName: string): string {
		return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatNumber(num: number): string {
		if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
		if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
		return num.toString();
	}

	function updateChart() {
		if (!chart) return;

		const data = $historyStore.data;
		if (data.length === 0) return;

		// Get labels (time)
		const labels = data.map(point => {
			const date = new Date(point.timestamp);
			return date.toLocaleTimeString('en-US', { hour12: false, minute: '2-digit', second: '2-digit' });
		});

		// Get values based on selected metric
		let values: number[];
		let label: string;
		let color: string;

		switch (selectedMetric) {
			case 'threats':
				values = data.map(p => p.threats);
				label = 'Threats Detected';
				color = getComputedColor('--error');
				break;
			case 'packets':
				values = data.map(p => p.packets);
				label = 'Packets Processed';
				color = getComputedColor('--accent-primary');
				break;
			case 'bandwidth':
				values = data.map(p => p.bandwidth);
				label = 'Bandwidth (B/s)';
				color = getComputedColor('--info');
				break;
		}

		chart.data.labels = labels;
		chart.data.datasets[0].data = values;
		chart.data.datasets[0].label = label;
		chart.data.datasets[0].borderColor = color;
		chart.data.datasets[0].backgroundColor = color + '20';
		chart.update('none');
	}

	function createChart() {
		if (!canvas) return;

		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		if (chart) {
			chart.destroy();
		}

		chart = new Chart(ctx, {
			type: 'line',
			data: {
				labels: [],
				datasets: [{
					label: 'Threats Detected',
					data: [],
					borderColor: getComputedColor('--error'),
					backgroundColor: getComputedColor('--error') + '20',
					borderWidth: 2,
					fill: true,
					tension: 0.4,
					pointRadius: 0,
					pointHoverRadius: 4
				}]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				interaction: {
					intersect: false,
					mode: 'index'
				},
				scales: {
					y: {
						beginAtZero: true,
						ticks: {
							color: getComputedColor('--text-secondary'),
							font: {
								family: "'JetBrains Mono', monospace",
								size: 11
							},
							callback: function(value) {
								if (selectedMetric === 'bandwidth') {
									return formatBytes(value as number);
								}
								return formatNumber(value as number);
							}
						},
						grid: {
							color: getComputedColor('--border-primary')
						}
					},
					x: {
						ticks: {
							color: getComputedColor('--text-secondary'),
							font: {
								family: "'JetBrains Mono', monospace",
								size: 10
							},
							maxRotation: 0,
							autoSkip: true,
							maxTicksLimit: 8
						},
						grid: {
							color: getComputedColor('--border-primary')
						}
					}
				},
				plugins: {
					legend: {
						display: false
					},
					tooltip: {
						backgroundColor: getComputedColor('--bg-tertiary'),
						titleColor: getComputedColor('--text-primary'),
						bodyColor: getComputedColor('--text-secondary'),
						borderColor: getComputedColor('--border-primary'),
						borderWidth: 1,
						padding: 12,
						bodyFont: {
							family: "'JetBrains Mono', monospace"
						},
						callbacks: {
							label: (context) => {
								const value = context.parsed.y;
								if (selectedMetric === 'bandwidth') {
									return ` ${formatBytes(value)}/s`;
								}
								return ` ${formatNumber(value)}`;
							}
						}
					}
				}
			}
		});

		updateChart();
	}

	function changeTimeRange(range: typeof timeRange) {
		timeRange = range;
		historyStore.setMaxPoints(timeRangePoints[range]);
	}

	onMount(() => {
		createChart();
	});

	onDestroy(() => {
		if (chart) {
			chart.destroy();
		}
	});

	$: if (chart && $historyStore) {
		updateChart();
	}

	$: if (chart && selectedMetric) {
		updateChart();
	}
</script>

<div class="chart-card card">
	<div class="chart-header">
		<h3>Activity Timeline</h3>
		<div class="controls">
			<div class="metric-selector">
				<button 
					class="metric-btn"
					class:active={selectedMetric === 'threats'}
					on:click={() => selectedMetric = 'threats'}
				>
					Threats
				</button>
				<button 
					class="metric-btn"
					class:active={selectedMetric === 'packets'}
					on:click={() => selectedMetric = 'packets'}
				>
					Packets
				</button>
				<button 
					class="metric-btn"
					class:active={selectedMetric === 'bandwidth'}
					on:click={() => selectedMetric = 'bandwidth'}
				>
					Bandwidth
				</button>
			</div>
			<div class="time-selector">
				{#each ['1m', '5m', '15m', '30m', '1h'] as range}
					<button 
						class="time-btn"
						class:active={timeRange === range}
						on:click={() => changeTimeRange(range)}
					>
						{range}
					</button>
				{/each}
			</div>
		</div>
	</div>
	<div class="chart-container">
		<canvas bind:this={canvas}></canvas>
	</div>
</div>

<style>
	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.chart-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1.25rem;
		flex-wrap: wrap;
		gap: 1rem;
	}

	h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0;
	}

	.controls {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
	}

	.metric-selector,
	.time-selector {
		display: flex;
		gap: 0.25rem;
		background: var(--bg-tertiary);
		padding: 0.25rem;
		border-radius: 0.5rem;
	}

	.metric-btn,
	.time-btn {
		padding: 0.5rem 0.875rem;
		border-radius: 0.375rem;
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.metric-btn:hover,
	.time-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.metric-btn.active,
	.time-btn.active {
		background: var(--accent-primary);
		color: var(--text-inverse);
	}

	.chart-container {
		position: relative;
		height: 300px;
	}

	@media (max-width: 768px) {
		.chart-header {
			flex-direction: column;
			align-items: flex-start;
		}

		.controls {
			width: 100%;
			flex-direction: column;
		}

		.metric-selector,
		.time-selector {
			width: 100%;
			justify-content: space-between;
		}
	}
</style>