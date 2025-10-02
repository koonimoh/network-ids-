<script lang="ts">
	import { onMount } from 'svelte';
	import { IPLookupService, type IPInfo, type AbuseIPDBInfo } from '$lib/services/ip-lookup';

	export let ip: string;
	export let compact = false;

	let loading = false;
	let ipInfo: IPInfo | null = null;
	let abuseInfo: AbuseIPDBInfo | null = null;
	let ipType: 'private' | 'loopback' | 'public' = 'public';
	let error: string | null = null;

	onMount(() => {
		lookupIP();
	});

	async function lookupIP() {
		loading = true;
		error = null;

		try {
			ipType = IPLookupService.getIPType(ip);

			// Only lookup public IPs
			if (ipType === 'public') {
				ipInfo = await IPLookupService.getIPInfo(ip);
				// Backend handles API key, so always attempt to fetch abuse info
				abuseInfo = await IPLookupService.getAbuseInfo(ip, '');
			}
		} catch (err: any) {
			error = err.message || 'Lookup failed';
		} finally {
			loading = false;
		}
	}

	function getThreatLevel(score: number): string {
		if (score >= 75) return 'critical';
		if (score >= 50) return 'high';
		if (score >= 25) return 'medium';
		return 'low';
	}

	$: location = ipInfo ? IPLookupService.formatLocation(ipInfo) : null;
	$: flag = ipInfo?.country_code ? IPLookupService.getCountryFlag(ipInfo.country_code) : '';
</script>

<div class="ip-lookup" class:compact>
	{#if loading}
		<div class="loading">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
				<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
			</svg>
			<span>Looking up IP...</span>
		</div>
	{:else if error}
		<div class="error">
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="10"/>
				<line x1="12" y1="8" x2="12" y2="12"/>
				<line x1="12" y1="16" x2="12.01" y2="16"/>
			</svg>
			<span>{error}</span>
		</div>
	{:else if ipType !== 'public'}
		<div class="ip-type-badge {ipType}">
			{ipType.toUpperCase()} IP
		</div>
	{:else if ipInfo}
		<div class="ip-info">
			{#if !compact}
				<div class="info-header">
					<h4>IP Intelligence</h4>
					<button class="refresh-btn" on:click={lookupIP} title="Refresh">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
						</svg>
					</button>
				</div>
			{/if}

			<div class="info-grid">
				{#if location}
					<div class="info-item">
						<span class="info-icon">📍</span>
						<div class="info-content">
							<span class="info-label">Location</span>
							<span class="info-value">{flag} {location}</span>
						</div>
					</div>
				{/if}

				{#if ipInfo.org}
					<div class="info-item">
						<span class="info-icon">🏢</span>
						<div class="info-content">
							<span class="info-label">Organization</span>
							<span class="info-value">{ipInfo.org}</span>
						</div>
					</div>
				{/if}

				{#if ipInfo.timezone}
					<div class="info-item">
						<span class="info-icon">🕐</span>
						<div class="info-content">
							<span class="info-label">Timezone</span>
							<span class="info-value">{ipInfo.timezone}</span>
						</div>
					</div>
				{/if}
			</div>

			{#if abuseInfo}
				<div class="abuse-info">
					<div class="abuse-header">
						<span class="abuse-label">Abuse Confidence Score</span>
						<span class="abuse-score {getThreatLevel(abuseInfo.abuseConfidenceScore)}">
							{abuseInfo.abuseConfidenceScore}%
						</span>
					</div>
					<div class="abuse-bar">
						<div 
							class="abuse-fill {getThreatLevel(abuseInfo.abuseConfidenceScore)}" 
							style="width: {abuseInfo.abuseConfidenceScore}%"
						></div>
					</div>
					
					{#if abuseInfo.totalReports > 0}
						<div class="abuse-details">
							<span>📊 {abuseInfo.totalReports} reports from {abuseInfo.numDistinctUsers} users</span>
							{#if abuseInfo.isWhitelisted}
								<span class="whitelisted">✓ Whitelisted</span>
							{/if}
						</div>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.ip-lookup {
		padding: 1rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
	}

	.ip-lookup.compact {
		padding: 0.75rem;
	}

	.loading,
	.error {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: var(--text-secondary);
		font-size: 0.875rem;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.error {
		color: var(--error);
	}

	.ip-type-badge {
		display: inline-block;
		padding: 0.5rem 1rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.375rem;
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

	.ip-type-badge.private {
		background: color-mix(in srgb, var(--info) 10%, transparent);
		border-color: var(--info);
		color: var(--info);
	}

	.info-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
	}

	h4 {
		font-size: 0.875rem;
		font-weight: 600;
		margin: 0;
	}

	.refresh-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.25rem;
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.refresh-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.info-grid {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.info-item {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
	}

	.info-icon {
		font-size: 1.25rem;
		flex-shrink: 0;
	}

	.info-content {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
		flex: 1;
	}

	.info-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.info-value {
		font-size: 0.875rem;
		color: var(--text-primary);
		line-height: 1.4;
	}

	.abuse-info {
		margin-top: 1rem;
		padding-top: 1rem;
		border-top: 1px solid var(--border-primary);
	}

	.abuse-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 0.5rem;
	}

	.abuse-label {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-secondary);
	}

	.abuse-score {
		font-size: 0.875rem;
		font-weight: 700;
		padding: 0.25rem 0.625rem;
		border-radius: 0.25rem;
	}

	.abuse-score.low {
		background: color-mix(in srgb, var(--success) 20%, transparent);
		color: var(--success);
	}

	.abuse-score.medium {
		background: color-mix(in srgb, var(--warning) 20%, transparent);
		color: var(--warning);
	}

	.abuse-score.high,
	.abuse-score.critical {
		background: color-mix(in srgb, var(--error) 20%, transparent);
		color: var(--error);
	}

	.abuse-bar {
		height: 0.5rem;
		background: var(--bg-secondary);
		border-radius: 0.25rem;
		overflow: hidden;
		margin-bottom: 0.5rem;
	}

	.abuse-fill {
		height: 100%;
		transition: width 0.3s ease;
	}

	.abuse-fill.low {
		background: var(--success);
	}

	.abuse-fill.medium {
		background: var(--warning);
	}

	.abuse-fill.high,
	.abuse-fill.critical {
		background: var(--error);
	}

	.abuse-details {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	.whitelisted {
		color: var(--success);
		font-weight: 600;
	}
</style>