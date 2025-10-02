<script lang="ts">
	import { notificationStore } from '$lib/stores/notifications';
	
	let requestingPermission = false;

	async function handleEnableNotifications() {
		if ($notificationStore.permission !== 'granted') {
			requestingPermission = true;
			const granted = await notificationStore.requestPermission();
			requestingPermission = false;
			
			if (granted) {
				notificationStore.toggleEnabled();
			} else {
				alert('Please allow notifications in your browser settings to enable this feature.');
			}
		} else {
			notificationStore.toggleEnabled();
		}
	}

	function getSeverityColor(severity: string): string {
		switch (severity) {
			case 'Critical': return 'var(--severity-critical)';
			case 'High': return 'var(--severity-high)';
			case 'Medium': return 'var(--severity-medium)';
			case 'Low': return 'var(--severity-low)';
			default: return 'var(--text-secondary)';
		}
	}
	
	function handleSeverityChange(severity: string) {
		notificationStore.setMinSeverity(severity as 'Low' | 'Medium' | 'High' | 'Critical');
	}
</script>

<div class="notification-settings card">
	<div class="settings-header">
		<div>
			<h3>Browser Notifications</h3>
			<p class="subtitle">Get desktop alerts for network threats</p>
		</div>
		
		<label class="toggle-switch">
			<input 
				type="checkbox" 
				checked={$notificationStore.enabled}
				on:change={handleEnableNotifications}
				disabled={requestingPermission}
			/>
			<span class="slider"></span>
		</label>
	</div>

	{#if $notificationStore.enabled}
		<div class="settings-content">
			<!-- Sound Toggle -->
			<div class="setting-row">
				<div class="setting-info">
					<label>Notification Sound</label>
					<span class="setting-description">Play alert sound for critical threats</span>
				</div>
				<label class="toggle-switch">
					<input 
						type="checkbox" 
						checked={$notificationStore.soundEnabled}
						on:change={() => notificationStore.toggleSound()}
					/>
					<span class="slider"></span>
				</label>
			</div>

			<!-- Minimum Severity -->
			<div class="setting-row">
				<div class="setting-info">
					<label>Minimum Severity</label>
					<span class="setting-description">Only notify for threats at or above this level</span>
				</div>
				<div class="severity-buttons">
					{#each ['Low', 'Medium', 'High', 'Critical'] as severity}
						<button
							class="severity-btn"
							class:active={$notificationStore.minSeverity === severity}
							style="--severity-color: {getSeverityColor(severity)}"
							on:click={() => handleSeverityChange(severity)}
						>
							{severity}
						</button>
					{/each}
				</div>
			</div>

			<!-- Permission Status -->
			{#if $notificationStore.permission === 'denied'}
				<div class="warning-message">
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<circle cx="12" cy="12" r="10"/>
						<line x1="12" y1="8" x2="12" y2="12"/>
						<line x1="12" y1="16" x2="12.01" y2="16"/>
					</svg>
					<div>
						<strong>Notifications Blocked</strong>
						<p>Please enable notifications in your browser settings for this site.</p>
					</div>
				</div>
			{/if}

			<!-- Test Notification -->
			<button 
				class="test-btn"
				on:click={() => {
					notificationStore.sendNotification({
						id: 'test-123',
						timestamp: new Date().toISOString(),
						severity: 'High',
						threat_type: 'Test Notification',
						confidence: 0.9,
						anomaly_score: 0.85,
						source_ip: '192.168.1.100',
						target_ip: null,
						affected_ports: [80, 443],
						description: 'This is a test notification to verify browser alerts are working.',
						explanation: {
							primary_indicators: ['Test'],
							feature_importance: {},
							similar_incidents: [],
							recommended_actions: []
						},
						raw_packets: []
					});
				}}
				disabled={$notificationStore.permission !== 'granted'}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
					<path d="M13.73 21a2 2 0 0 1-3.46 0"/>
				</svg>
				Send Test Notification
			</button>
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

	.settings-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
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

	.toggle-switch {
		position: relative;
		display: inline-block;
		width: 3rem;
		height: 1.75rem;
	}

	.toggle-switch input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.slider {
		position: absolute;
		cursor: pointer;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		transition: 0.3s;
		border-radius: 2rem;
	}

	.slider:before {
		position: absolute;
		content: "";
		height: 1.25rem;
		width: 1.25rem;
		left: 0.25rem;
		bottom: 0.2rem;
		background: var(--text-secondary);
		transition: 0.3s;
		border-radius: 50%;
	}

	input:checked + .slider {
		background: var(--accent-primary);
		border-color: var(--accent-primary);
	}

	input:checked + .slider:before {
		background: white;
		transform: translateX(1.25rem);
	}

	input:disabled + .slider {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.settings-content {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		padding-top: 1rem;
		border-top: 1px solid var(--border-primary);
	}

	.setting-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.setting-info {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.setting-info label {
		font-size: 0.9375rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	.setting-description {
		font-size: 0.8125rem;
		color: var(--text-secondary);
	}

	.severity-buttons {
		display: flex;
		gap: 0.5rem;
	}

	.severity-btn {
		padding: 0.5rem 0.875rem;
		border-radius: 0.375rem;
		font-size: 0.8125rem;
		font-weight: 600;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.severity-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.severity-btn.active {
		background: color-mix(in srgb, var(--severity-color) 20%, transparent);
		border-color: var(--severity-color);
		color: var(--severity-color);
	}

	.warning-message {
		display: flex;
		gap: 0.75rem;
		padding: 1rem;
		background: color-mix(in srgb, var(--warning) 10%, transparent);
		border: 1px solid var(--warning);
		border-radius: 0.5rem;
	}

	.warning-message svg {
		flex-shrink: 0;
		color: var(--warning);
	}

	.warning-message strong {
		display: block;
		font-size: 0.875rem;
		margin-bottom: 0.25rem;
		color: var(--text-primary);
	}

	.warning-message p {
		font-size: 0.8125rem;
		color: var(--text-secondary);
		margin: 0;
	}

	.test-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.75rem 1.25rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-primary);
		transition: all 0.2s;
	}

	.test-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		border-color: var(--border-secondary);
	}

	.test-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	@media (max-width: 768px) {
		.setting-row {
			flex-direction: column;
			align-items: flex-start;
		}

		.severity-buttons {
			width: 100%;
			display: grid;
			grid-template-columns: repeat(2, 1fr);
		}
	}
</style>