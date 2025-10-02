<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Chart, DoughnutController, ArcElement, Tooltip, Legend } from 'chart.js';
	import type { SystemStats } from '$lib/types';

	Chart.register(DoughnutController, ArcElement, Tooltip, Legend);

	export let stats: SystemStats;

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;

	function getComputedColor(varName: string): string {
		return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
	}

	function updateChart() {
		if (!chart) return;

		const protocols = Object.entries(stats.protocol_distribution);
		const labels = protocols.map(([proto]) => proto);
		const data = protocols.map(([, count]) => count);

		chart.data.labels = labels;
		chart.data.datasets[0].data = data;
		chart.update('none');
	}

	onMount(() => {
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const protocols = Object.entries(stats.protocol_distribution);
		const labels = protocols.map(([proto]) => proto);
		const data = protocols.map(([, count]) => count);

		chart = new Chart(ctx, {
			type: 'doughnut',
			data: {
				labels,
				datasets: [{
					data,
					backgroundColor: [
						getComputedColor('--accent-primary'),
						getComputedColor('--info'),
						getComputedColor('--success'),
						getComputedColor('--warning'),
						getComputedColor('--error')
					],
					borderWidth: 2,
					borderColor: getComputedColor('--bg-secondary')
				}]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: {
						position: 'bottom',
						labels: {
							color: getComputedColor('--text-secondary'),
							font: {
								family: "'JetBrains Mono', monospace",
								size: 12
							},
							padding: 16
						}
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
								const label = context.label || '';
								const value = context.parsed;
								return ` ${label}: ${value.toLocaleString()}`;
							}
						}
					}
				}
			}
		});
	});

	onDestroy(() => {
		if (chart) {
			chart.destroy();
		}
	});

	$: if (chart && stats) {
		updateChart();
	}
</script>

<div class="chart-card card">
	<h3>Protocol Distribution</h3>
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

	h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 1.25rem;
	}

	.chart-container {
		position: relative;
		height: 300px;
	}
</style>