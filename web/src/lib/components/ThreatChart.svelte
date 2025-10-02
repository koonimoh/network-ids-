<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Chart, BarController, CategoryScale, LinearScale, BarElement, Tooltip, Legend } from 'chart.js';
	import type { SystemStats } from '$lib/types';

	Chart.register(BarController, CategoryScale, LinearScale, BarElement, Tooltip, Legend);

	export let stats: SystemStats;

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;

	function getComputedColor(varName: string): string {
		return getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
	}

	function updateChart() {
		if (!chart) return;

		const severities = ['Low', 'Medium', 'High', 'Critical'];
		const data = severities.map(sev => stats.alert_counts[sev] || 0);

		chart.data.datasets[0].data = data;
		chart.update('none');
	}

	onMount(() => {
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const severities = ['Low', 'Medium', 'High', 'Critical'];
		const data = severities.map(sev => stats.alert_counts[sev] || 0);

		chart = new Chart(ctx, {
			type: 'bar',
			data: {
				labels: severities,
				datasets: [{
					label: 'Threats by Severity',
					data,
					backgroundColor: [
						getComputedColor('--severity-low'),
						getComputedColor('--severity-medium'),
						getComputedColor('--severity-high'),
						getComputedColor('--severity-critical')
					],
					borderWidth: 0,
					borderRadius: 6
				}]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				scales: {
					y: {
						beginAtZero: true,
						ticks: {
							color: getComputedColor('--text-secondary'),
							font: {
								family: "'JetBrains Mono', monospace",
								size: 11
							},
							precision: 0
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
								size: 11,
								weight: 600
							}
						},
						grid: {
							display: false
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
								return ` Count: ${context.parsed.y}`;
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
	<h3>Threats by Severity</h3>
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