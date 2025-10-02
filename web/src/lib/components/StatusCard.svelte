<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { SystemStatus } from '$lib/types';

	export let status: SystemStatus | null;
	export let loading: boolean;

	const dispatch = createEventDispatcher();

	function handleStart() {
		dispatch('start');
	}

	function handleStop() {
		dispatch('stop');
	}
</script>

<div class="status-card card">
	<div class="status-header">
		<div class="status-info">
			<h3>System Status</h3>
			{#if status}
				<div class="status-badge" class:running={status.running}>
					<div class="status-dot"></div>
					<span>{status.running ? 'RUNNING' : 'STOPPED'}</span>
				</div>
			{/if}
		</div>
		
		<div class="status-actions">
			{#if status?.running}
				<button 
					class="btn btn-danger" 
					on:click={handleStop}
					disabled={loading}
				>
					{loading ? 'STOPPING...' : 'STOP'}
				</button>
			{:else}
				<button 
					class="btn btn-primary" 
					on:click={handleStart}
					disabled={loading}
				>
					{loading ? 'STARTING...' : 'START'}
				</button>
			{/if}
		</div>
	</div>

	{#if status}
		<div class="status-meta">
			<div class="meta-item">
				<span class="meta-label">VERSION</span>
				<span class="meta-value mono">{status.version}</span>
			</div>
			{#if status.running}
				<div class="meta-item">
					<span class="meta-label">UPTIME</span>
					<span class="meta-value mono">
						{Math.floor(status.uptime_seconds / 3600)}h {Math.floor((status.uptime_seconds % 3600) / 60)}m
					</span>
				</div>
			{/if}
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

	.status-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1.25rem;
	}

	.status-info h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 0.75rem;
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.375rem 0.875rem;
		border-radius: 0.375rem;
		background: var(--bg-tertiary);
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

	.status-badge.running {
		background: rgba(34, 197, 94, 0.1);
		color: var(--success);
	}

	.status-dot {
		width: 0.5rem;
		height: 0.5rem;
		border-radius: 50%;
		background: var(--text-tertiary);
	}

	.status-badge.running .status-dot {
		background: var(--success);
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}

	.btn {
		padding: 0.625rem 1.25rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		letter-spacing: 0.025em;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--accent-primary);
		color: var(--text-inverse);
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.btn-danger {
		background: var(--error);
		color: var(--text-inverse);
	}

	.btn-danger:hover:not(:disabled) {
		opacity: 0.9;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.status-meta {
		display: flex;
		gap: 2rem;
		padding-top: 1.25rem;
		border-top: 1px solid var(--border-primary);
	}

	.meta-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.meta-label {
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		color: var(--text-tertiary);
	}

	.meta-value {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	@media (max-width: 640px) {
		.status-header {
			flex-direction: column;
			align-items: flex-start;
			gap: 1rem;
		}

		.status-actions {
			width: 100%;
		}

		.btn {
			width: 100%;
		}
	}
</style>