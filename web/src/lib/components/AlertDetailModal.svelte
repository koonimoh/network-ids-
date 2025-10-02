<script lang="ts">
	import type { ThreatAlert } from '$lib/types';
	import { createEventDispatcher } from 'svelte';
	import IPLookup from './IPLookup.svelte';
	import { alertStatusStore } from '$lib/stores/alertStatus';
	import type { AlertStatus } from '$lib/types';

	export let alert: ThreatAlert;
	export let open = false;
	let blocking = false;
	let blockSuccess = false;

	// Subscribe to the store to make it reactive
	$: alertKey = `${alert.source_ip}-${alert.threat_type}`;
	$: currentStatus = $alertStatusStore.acknowledgments[alertKey]?.status || 'new';

	const dispatch = createEventDispatcher();

	function close() {
		dispatch('close');
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
	}

	function formatTimestamp(timestamp: string): string {
		return new Date(timestamp).toLocaleString('en-US', {
			dateStyle: 'medium',
			timeStyle: 'medium'
		});
	}
	
	
	async function blockIP() {  
		if (!confirm(`Block IP ${alert.source_ip}?\n\nReason: ${alert.threat_type}`)) return;

		blocking = true;
		try {
			const res = await fetch('/api/blocklist', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					ip: alert.source_ip,
					reason: alert.threat_type,
					notes: alert.description,
					expires_in_hours: null
				})
			});

			const data = await res.json();
			if (data.success) {
				blockSuccess = true;
				setTimeout(() => blockSuccess = false, 3000);
			} else {
				alert('Failed to block IP: ' + (data.error || 'Unknown error'));
			}
		} catch (err) {
			console.error('Failed to block IP:', err);
			alert('Failed to block IP');
		} finally {
			blocking = false;
		}
	}
	
	
	
	function setStatus(status: AlertStatus) {
		alertStatusStore.setStatus(alert, status);
	}

	function getStatusLabel(status: AlertStatus): string {
		switch (status) {
			case 'new': return 'New';
			case 'reviewed': return 'Reviewed';
			case 'investigating': return 'Investigating';
			case 'resolved': return 'Resolved';
			case 'false_positive': return 'False Positive';
		}
	}
	
	
	
	
	
	$: sortedFeatures = Object.entries(alert.explanation.feature_importance || {})
		.sort((a, b) => b[1] - a[1]);
</script>

{#if open}
	<div class="modal-backdrop" on:click={handleBackdropClick}>
		<div class="modal">
			<div class="modal-header">
				<div class="header-left">
					<div class="severity-badge" style="--severity-color: var(--severity-{alert.severity.toLowerCase()})">
						{alert.severity.toUpperCase()}
					</div>
					<h2>{alert.threat_type}</h2>
				</div>
				<div class="header-actions">
					{#if blockSuccess}
						<span class="success-badge">✓ Blocked</span>
					{:else}
						<button class="btn-block" on:click={blockIP} disabled={blocking}>
							<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="10"/>
								<line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/>
							</svg>
							{blocking ? 'Blocking...' : 'Block IP'}
						</button>
					{/if}
					<button class="close-btn" on:click={close}>
						<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<line x1="18" y1="6" x2="6" y2="18"/>
							<line x1="6" y1="6" x2="18" y2="18"/>
						</svg>
					</button>
				</div>
			</div>

			<div class="modal-content">
				<div class="info-grid">
					<div class="info-card">
						<h3>Source Information</h3>
						<div class="info-row">
							<span class="label">Source IP</span>
							<div class="value-with-action">
								<span class="mono">{alert.source_ip}</span>
								<button class="icon-btn" on:click={() => copyToClipboard(alert.source_ip)} title="Copy">
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
										<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
									</svg>
								</button>
							</div>
						</div>
						{#if alert.target_ip}
							<div class="info-row">
								<span class="label">Target IP</span>
								<div class="value-with-action">
									<span class="mono">{alert.target_ip}</span>
									<button class="icon-btn" on:click={() => copyToClipboard(alert.target_ip)} title="Copy">
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
											<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
											<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
										</svg>
									</button>
								</div>
							</div>
						{/if}
						{#if alert.affected_ports.length > 0}
							<div class="info-row">
								<span class="label">Affected Ports</span>
								<div class="port-tags">
									{#each alert.affected_ports.slice(0, 10) as port}
										<span class="port-tag mono">{port}</span>
									{/each}
									{#if alert.affected_ports.length > 10}
										<span class="port-tag">+{alert.affected_ports.length - 10} more</span>
									{/if}
								</div>
							</div>
						{/if}
					</div>

					<div class="info-card">
						<h3>Detection Metrics</h3>
						<div class="info-row">
							<span class="label">Confidence</span>
							<div class="metric-bar">
								<div class="metric-fill" style="width: {alert.confidence * 100}%"></div>
								<span class="metric-value">{(alert.confidence * 100).toFixed(1)}%</span>
							</div>
						</div>
						<div class="info-row">
							<span class="label">Anomaly Score</span>
							<div class="metric-bar">
								<div class="metric-fill anomaly" style="width: {alert.anomaly_score * 100}%"></div>
								<span class="metric-value">{(alert.anomaly_score * 100).toFixed(1)}%</span>
							</div>
						</div>
						<div class="info-row">
							<span class="label">Timestamp</span>
							<span class="value mono">{formatTimestamp(alert.timestamp)}</span>
						</div>
						<div class="info-row">
							<span class="label">Alert ID</span>
							<span class="value mono small">{alert.id}</span>
						</div>
					</div>
				
				
				<div class="info-card">			
					<h3>Alert Status</h3>
					<div class="status-buttons">
						<button
							class="status-btn"
							class:active={currentStatus === 'new'}
							on:click={() => setStatus('new')}
						>
							New
						</button>
						<button
							class="status-btn"
							class:active={currentStatus === 'reviewed'}
							on:click={() => setStatus('reviewed')}
						>
							Reviewed
						</button>
						<button
							class="status-btn"
							class:active={currentStatus === 'investigating'}
							on:click={() => setStatus('investigating')}
						>
							Investigating
						</button>
						<button 
							class="status-btn"
							class:active={currentStatus === 'resolved'}
							on:click={() => setStatus('resolved')}
						>
							Resolved
						</button>
						<button
							class="status-btn"
							class:active={currentStatus === 'false_positive'}
							on:click={() => setStatus('false_positive')}
						>
							False Positive
						</button>
						</div>
						<p class="status-help">
							Mark this alert's investigation status
						</p>
					</div>
				</div>
				
				

				<div class="section">
					<h3>IP Intelligence</h3>
					<IPLookup ip={alert.source_ip} />
				</div>

				<div class="section">
					<h3>Description</h3>
					<p class="description">{alert.description}</p>
				</div>

				<div class="section">
					<h3>Primary Indicators</h3>
					<ul class="indicator-list">
						{#each alert.explanation.primary_indicators as indicator}
							<li>{indicator}</li>
						{/each}
					</ul>
				</div>

				{#if sortedFeatures.length > 0}
					<div class="section">
						<h3>Feature Importance</h3>
						<div class="features-list">
							{#each sortedFeatures as [feature, importance]}
								<div class="feature-item">
									<span class="feature-name">{feature}</span>
									<div class="feature-bar">
										<div class="feature-fill" style="width: {importance * 100}%"></div>
										<span class="feature-value">{(importance * 100).toFixed(0)}%</span>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if alert.explanation.similar_incidents.length > 0}
					<div class="section">
						<h3>Similar Incidents</h3>
						<ul class="incident-list">
							{#each alert.explanation.similar_incidents as incident}
								<li>{incident}</li>
							{/each}
						</ul>
					</div>
				{/if}

				<div class="section">
					<h3>Recommended Actions</h3>
					<div class="actions-checklist">
						{#each alert.explanation.recommended_actions as action}
							<label class="action-item">
								<input type="checkbox" />
								<span>{action}</span>
							</label>
						{/each}
					</div>
				</div>
				

				{#if alert.raw_packets.length > 0}
					<div class="section">
						<h3>Associated Packets ({alert.raw_packets.length})</h3>
						<div class="packet-ids">
							{#each alert.raw_packets.slice(0, 5) as packetId}
								<code class="packet-id">{packetId}</code>
							{/each}
							{#if alert.raw_packets.length > 5}
								<span class="more-packets">+{alert.raw_packets.length - 5} more packets</span>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: 2rem;
		backdrop-filter: blur(4px);
	}

	.modal {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		max-width: 900px;
		width: 100%;
		max-height: 90vh;
		display: flex;
		flex-direction: column;
		box-shadow: var(--shadow-lg);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.5rem;
		border-bottom: 1px solid var(--border-primary);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.severity-badge {
		padding: 0.375rem 0.75rem;
		border-radius: 0.375rem;
		font-size: 0.75rem;
		font-weight: 700;
		letter-spacing: 0.05em;
		background: color-mix(in srgb, var(--severity-color) 20%, transparent);
		color: var(--severity-color);
	}

	h2 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		border-radius: 0.375rem;
		background: transparent;
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.close-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.modal-content {
		overflow-y: auto;
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.info-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 1rem;
	}

	.info-card {
		padding: 1rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
	}

	.info-card h3 {
		font-size: 0.875rem;
		font-weight: 600;
		margin-bottom: 1rem;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.info-row {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 0.75rem;
	}

	.info-row:last-child {
		margin-bottom: 0;
	}

	.label {
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.value {
		font-size: 0.875rem;
		color: var(--text-primary);
	}

	.value.small {
		font-size: 0.75rem;
	}

	.value-with-action {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		border-radius: 0.25rem;
		background: transparent;
		color: var(--text-tertiary);
		transition: all 0.2s;
	}

	.icon-btn:hover {
		background: var(--bg-hover);
		color: var(--accent-primary);
	}

	.port-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 0.375rem;
	}

	.port-tag {
		padding: 0.25rem 0.5rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.25rem;
		font-size: 0.75rem;
	}

	.metric-bar {
		position: relative;
		height: 2rem;
		background: var(--bg-secondary);
		border-radius: 0.375rem;
		overflow: hidden;
	}

	.metric-fill {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: var(--accent-primary);
		transition: width 0.3s ease;
	}

	.metric-fill.anomaly {
		background: var(--warning);
	}

	.metric-value {
		position: absolute;
		top: 50%;
		right: 0.75rem;
		transform: translateY(-50%);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.section h3 {
		font-size: 1rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.description {
		font-size: 0.875rem;
		line-height: 1.6;
		color: var(--text-secondary);
	}

	.indicator-list,
	.incident-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.indicator-list li,
	.incident-list li {
		font-size: 0.875rem;
		color: var(--text-secondary);
		padding-left: 1.5rem;
		position: relative;
	}

	.indicator-list li::before {
		content: '⚠';
		position: absolute;
		left: 0;
		color: var(--warning);
	}

	.incident-list li::before {
		content: '•';
		position: absolute;
		left: 0.5rem;
		color: var(--accent-primary);
	}

	.features-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.feature-item {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.feature-name {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	.feature-bar {
		position: relative;
		height: 1.5rem;
		background: var(--bg-tertiary);
		border-radius: 0.25rem;
		overflow: hidden;
	}

	.feature-fill {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: linear-gradient(90deg, var(--accent-primary), var(--info));
		transition: width 0.3s ease;
	}

	.feature-value {
		position: absolute;
		top: 50%;
		right: 0.5rem;
		transform: translateY(-50%);
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.actions-checklist {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.action-item {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		cursor: pointer;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border-radius: 0.375rem;
		transition: all 0.2s;
	}

	.action-item:hover {
		background: var(--bg-hover);
	}

	.action-item input[type="checkbox"] {
		margin-top: 0.125rem;
		width: 1.125rem;
		height: 1.125rem;
		cursor: pointer;
	}

	.action-item span {
		font-size: 0.875rem;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.packet-ids {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		align-items: center;
	}

	.packet-id {
		padding: 0.375rem 0.625rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-family: 'JetBrains Mono', monospace;
		color: var(--accent-primary);
	}

	.more-packets {
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	@media (max-width: 768px) {
		.modal-backdrop {
			padding: 0;
		}

		.modal {
			max-height: 100vh;
			border-radius: 0;
		}

		.info-grid {
			grid-template-columns: 1fr;
		}
	}
	
	
	
	.header-actions {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.btn-block {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: var(--error);
		color: var(--text-inverse);
		border-radius: 0.375rem;
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.btn-block:hover:not(:disabled) {
		background: color-mix(in srgb, var(--error) 80%, black);
	}

	.btn-block:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.success-badge {
		padding: 0.5rem 1rem;
		background: color-mix(in srgb, var(--success) 20%, transparent);
		color: var(--success);
		border-radius: 0.375rem;
		font-size: 0.875rem;
		font-weight: 600;
	}
	
	.status-buttons {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 0.5rem;
		margin-bottom: 0.75rem;
	}

	.status-btn {
		padding: 0.625rem 0.75rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.375rem;
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-secondary);
		transition: all 0.2s;
		text-align: center;
	}

	.status-btn:hover {
		background: var(--bg-hover);
		border-color: var(--border-secondary);
		color: var(--text-primary);
	}

	.status-btn.active {
		background: var(--accent-primary);
		border-color: var(--accent-primary);
		color: var(--text-inverse);
	}

	.status-help {
		font-size: 0.75rem;
		color: var(--text-tertiary);
		font-style: italic;
	}
</style>