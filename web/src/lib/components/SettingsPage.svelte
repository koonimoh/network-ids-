<script lang="ts">
	import { onMount } from 'svelte';
	import type { SystemConfig, ApiResponse } from '$lib/types';
	import NotificationSettings from './NotificationSettings.svelte'; 

	let config: SystemConfig | null = null;
	let loading = false;
	let saving = false;
	let successMessage = '';
	let errorMessage = '';

	async function loadConfig() {
		loading = true;
		try {
			const res = await fetch('/api/config');
			const data: ApiResponse<SystemConfig> = await res.json();
			if (data.success && data.data) {
				config = data.data;
			}
		} catch (err) {
			console.error('Failed to load config:', err);
			errorMessage = 'Failed to load configuration';
		} finally {
			loading = false;
		}
	}

	async function saveConfig() {
		if (!config) return;

		saving = true;
		successMessage = '';
		errorMessage = '';

		try {
			const res = await fetch('/api/config', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(config)
			});
			
			const data: ApiResponse<string> = await res.json();
			
			if (data.success) {
				successMessage = 'Configuration saved successfully';
				setTimeout(() => successMessage = '', 3000);
			} else {
				errorMessage = data.error || 'Failed to save configuration';
			}
		} catch (err) {
			console.error('Failed to save config:', err);
			errorMessage = 'Failed to save configuration';
		} finally {
			saving = false;
		}
	}

	async function resetConfig() {
		if (!confirm('Reset configuration to defaults?')) return;

		loading = true;
		try {
			const res = await fetch('/api/config');
			const data: ApiResponse<SystemConfig> = await res.json();
			if (data.success && data.data) {
				config = data.data;
				successMessage = 'Configuration reset to defaults';
				setTimeout(() => successMessage = '', 3000);
			}
		} catch (err) {
			console.error('Failed to reset config:', err);
			errorMessage = 'Failed to reset configuration';
		} finally {
			loading = false;
		}
	}


	onMount(() => {
		loadConfig();
	});
</script>

<div class="settings-page">
	<div class="page-header">
		<div>
			<h1>System Settings</h1>
			<p class="subtitle">Configure IDS behavior and detection parameters</p>
		</div>
		<div class="header-actions">
			<button class="btn-secondary" on:click={resetConfig} disabled={loading || saving}>
				Reset to Defaults
			</button>
			<button class="btn-primary" on:click={saveConfig} disabled={loading || saving || !config}>
				{#if saving}
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
						<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
					</svg>
					Saving...
				{:else}
					Save Changes
				{/if}
			</button>
		</div>
	</div>

	{#if successMessage}
		<div class="alert alert-success">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
				<polyline points="22 4 12 14.01 9 11.01"/>
			</svg>
			{successMessage}
		</div>
	{/if}

	{#if errorMessage}
		<div class="alert alert-error">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="10"/>
				<line x1="15" y1="9" x2="9" y2="15"/>
				<line x1="9" y1="9" x2="15" y2="15"/>
			</svg>
			{errorMessage}
		</div>
	{/if}

	{#if loading}
		<div class="loading-state">
			<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
				<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
			</svg>
			<p>Loading configuration...</p>
		</div>
	{:else if config}
		<div class="settings-grid">
			<div class="settings-section card">
				<h3>Network Configuration</h3>
				<div class="form-group">
					<label for="interface">Network Interface</label>
					<input 
						id="interface"
						type="text" 
						bind:value={config.interface}
						placeholder="e.g., Wi-Fi, eth0"
					/>
					<span class="help-text">Network interface to monitor for traffic</span>
				</div>

				<div class="form-group">
					<label for="max-pps">Maximum Packets Per Second</label>
					<input 
						id="max-pps"
						type="number" 
						bind:value={config.max_pps}
						min="100"
						max="1000000"
					/>
					<span class="help-text">Maximum packet processing rate limit</span>
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={config.use_simulation} />
						<span>Use Simulation Mode</span>
					</label>
					<span class="help-text">Enable simulated traffic for testing (auto-enabled on Windows)</span>
				</div>
			</div>

			<div class="settings-section card">
				<h3>Detection Parameters</h3>
				<div class="form-group">
					<label for="sensitivity">Detection Sensitivity</label>
					<div class="slider-container">
						<input 
							id="sensitivity"
							type="range" 
							bind:value={config.sensitivity}
							min="0"
							max="1"
							step="0.1"
						/>
						<span class="slider-value mono">{config.sensitivity.toFixed(1)}</span>
					</div>
					<span class="help-text">Higher values increase detection sensitivity</span>
				</div>

				<div class="form-group">
					<label for="anomaly-threshold">Anomaly Threshold</label>
					<div class="slider-container">
						<input 
							id="anomaly-threshold"
							type="range" 
							bind:value={config.alert_thresholds.anomaly_threshold}
							min="0"
							max="1"
							step="0.05"
						/>
						<span class="slider-value mono">{config.alert_thresholds.anomaly_threshold.toFixed(2)}</span>
					</div>
					<span class="help-text">Minimum anomaly score to trigger alerts</span>
				</div>

				<div class="form-group">
					<label for="min-confidence">Minimum Confidence</label>
					<div class="slider-container">
						<input 
							id="min-confidence"
							type="range" 
							bind:value={config.alert_thresholds.min_confidence}
							min="0"
							max="1"
							step="0.05"
						/>
						<span class="slider-value mono">{config.alert_thresholds.min_confidence.toFixed(2)}</span>
					</div>
					<span class="help-text">Minimum confidence level for threat alerts</span>
				</div>

				<div class="form-group">
					<label for="max-alerts">Max Alerts Per Minute</label>
					<input 
						id="max-alerts"
						type="number" 
						bind:value={config.alert_thresholds.max_alerts_per_minute}
						min="1"
						max="100"
					/>
					<span class="help-text">Rate limit for alert generation</span>
				</div>
			</div>

			<div class="settings-section card">
				<h3>Machine Learning</h3>
				<div class="form-group">
					<label for="update-freq">Model Update Frequency (seconds)</label>
					<input 
						id="update-freq"
						type="number" 
						bind:value={config.ml_config.update_frequency}
						min="60"
						max="3600"
					/>
					<span class="help-text">How often to retrain the ML model</span>
				</div>

				<div class="form-group">
					<label for="batch-size">Training Batch Size</label>
					<input 
						id="batch-size"
						type="number" 
						bind:value={config.ml_config.batch_size}
						min="32"
						max="512"
					/>
					<span class="help-text">Number of samples per training batch</span>
				</div>

				<div class="form-group">
					<label for="learning-rate">Learning Rate</label>
					<input 
						id="learning-rate"
						type="number" 
						bind:value={config.ml_config.learning_rate}
						min="0.0001"
						max="0.1"
						step="0.0001"
					/>
					<span class="help-text">ML model training learning rate</span>
				</div>

				<div class="form-group">
					<label for="window-size">Feature Window Size</label>
					<input 
						id="window-size"
						type="number" 
						bind:value={config.ml_config.window_size}
						min="10"
						max="1000"
					/>
					<span class="help-text">Number of packets to aggregate for features</span>
				</div>
			</div>
				<NotificationSettings />
		</div>
	{/if}
</div>

<style>
	.settings-page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		flex-wrap: wrap;
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

	.header-actions {
		display: flex;
		gap: 0.75rem;
	}

	.btn-primary, .btn-secondary {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.625rem 1.25rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--accent-primary);
		color: var(--text-inverse);
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.btn-secondary {
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-primary);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--bg-hover);
		border-color: var(--border-secondary);
	}

	.btn-primary:disabled, .btn-secondary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.alert {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 1rem 1.25rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.alert-success {
		background: color-mix(in srgb, var(--success) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--success) 30%, transparent);
		color: var(--success);
	}

	.alert-error {
		background: color-mix(in srgb, var(--error) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--error) 30%, transparent);
		color: var(--error);
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 4rem 2rem;
		color: var(--text-secondary);
	}

	.loading-state svg {
		margin-bottom: 1rem;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.settings-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		gap: 1.5rem;
	}

	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.settings-section h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 1.5rem;
	}

	.section-description {
		font-size: 0.875rem;
		color: var(--text-secondary);
		margin-bottom: 1.25rem;
		line-height: 1.5;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1.25rem;
	}

	.form-group:last-child {
		margin-bottom: 0;
	}

	label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	input[type="text"],
	input[type="number"] {
		padding: 0.625rem 0.875rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
	}

	input[type="range"] {
		width: 100%;
		height: 0.375rem;
		border-radius: 0.25rem;
		background: var(--border-primary);
		appearance: none;
		cursor: pointer;
	}

	input[type="range"]::-webkit-slider-thumb {
		appearance: none;
		width: 1.25rem;
		height: 1.25rem;
		border-radius: 50%;
		background: var(--accent-primary);
		cursor: pointer;
	}

	input[type="range"]::-moz-range-thumb {
		width: 1.25rem;
		height: 1.25rem;
		border-radius: 50%;
		background: var(--accent-primary);
		border: none;
		cursor: pointer;
	}

	.slider-container {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.slider-value {
		min-width: 3rem;
		text-align: right;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--accent-primary);
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		cursor: pointer;
	}

	.checkbox-label input[type="checkbox"] {
		width: 1.125rem;
		height: 1.125rem;
		cursor: pointer;
	}

	.help-text {
		font-size: 0.75rem;
		color: var(--text-tertiary);
		line-height: 1.4;
	}

	.help-text a {
		color: var(--accent-primary);
		text-decoration: none;
	}

	.help-text a:hover {
		text-decoration: underline;
	}

	.key-input-group {
		display: flex;
		gap: 0.5rem;
	}

	.key-input-group input {
		flex: 1;
	}

	@media (max-width: 768px) {
		.settings-grid {
			grid-template-columns: 1fr;
		}

		.page-header {
			flex-direction: column;
			align-items: flex-start;
		}

		.header-actions {
			width: 100%;
		}

		.btn-primary, .btn-secondary {
			flex: 1;
		}
	}
</style>