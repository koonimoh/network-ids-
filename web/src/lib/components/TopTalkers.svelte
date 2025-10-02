<script lang="ts">
	import type { SystemStats } from '$lib/types';

	export let stats: SystemStats;

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

	$: sortedTalkers = [...stats.top_talkers]
		.sort((a, b) => b[1] - a[1])
		.slice(0, 10);
</script>

<div class="top-talkers card">
	<div class="header">
		<h3>Top Network Talkers</h3>
		<span class="count">{sortedTalkers.length} active</span>
	</div>

	<div class="talkers-list">
		{#each sortedTalkers as [ip, bytes], index}
			<div class="talker-item">
				<div class="talker-rank">{index + 1}</div>
				<div class="talker-info">
					<span class="talker-ip mono">{ip}</span>
					<div class="talker-bar">
						<div 
							class="talker-bar-fill" 
							style="width: {(bytes / sortedTalkers[0][1]) * 100}%"
						></div>
					</div>
				</div>
				<span class="talker-bytes mono">{formatBytes(bytes)}</span>
			</div>
		{/each}
	</div>
</div>

<style>
	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1.25rem;
	}

	h3 {
		font-size: 1.125rem;
		font-weight: 600;
	}

	.count {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-secondary);
		padding: 0.25rem 0.625rem;
		background: var(--bg-tertiary);
		border-radius: 0.25rem;
	}

	.talkers-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.talker-item {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
		transition: all 0.2s;
	}

	.talker-item:hover {
		background: var(--bg-hover);
	}

	.talker-rank {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		border-radius: 0.375rem;
		background: var(--accent-primary);
		color: var(--text-inverse);
		font-size: 0.75rem;
		font-weight: 700;
		flex-shrink: 0;
	}

	.talker-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.talker-ip {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	.talker-bar {
		height: 0.375rem;
		background: var(--border-primary);
		border-radius: 0.25rem;
		overflow: hidden;
	}

	.talker-bar-fill {
		height: 100%;
		background: linear-gradient(90deg, var(--accent-primary), var(--info));
		transition: width 0.3s ease;
	}

	.talker-bytes {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-secondary);
		flex-shrink: 0;
	}
</style>