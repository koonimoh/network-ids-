export interface IPInfo {
	ip: string;
	city?: string;
	region?: string;
	country?: string;
	country_code?: string;
	loc?: string; // lat,lon
	org?: string; // Organization/ISP
	postal?: string;
	timezone?: string;
}

export interface AbuseIPDBInfo {
	abuseConfidenceScore: number;
	usageType: string;
	isp: string;
	domain: string;
	countryCode: string;
	isWhitelisted: boolean;
	totalReports: number;
	numDistinctUsers: number;
	lastReportedAt: string | null;
}

export class IPLookupService {
	/**
	 * Get IP information from ipinfo.io (free, no key required up to 50k/month)
	 */
	static async getIPInfo(ip: string): Promise<IPInfo | null> {
		try {
			const response = await fetch(`https://ipinfo.io/${ip}/json`, {
				headers: {
					'Accept': 'application/json'
				}
			});

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			}

			const data = await response.json();
			return data;
		} catch (error) {
			console.error('Failed to fetch IP info:', error);
			return null;
		}
	}

	/**
	 * Get abuse confidence score from AbuseIPDB via backend proxy
	 */
	static async getAbuseInfo(ip: string, apiKey: string): Promise<AbuseIPDBInfo | null> {
		// apiKey parameter ignored - backend handles the key
		
		try {
			const response = await fetch(
				`http://localhost:3000/api/ip-lookup/${ip}`
			);

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			}

			const result = await response.json();
			// Backend wraps in ApiResponse { success, data }
			// AbuseIPDB wraps in { data: {...} }
			// So we need result.data.data
			return result.data?.data || null;
		} catch (error) {
			console.error('Failed to fetch abuse info:', error);
			return null;
		}
	}

	/**
	 * Check if IP is in common ranges (private, loopback, etc)
	 */
	static getIPType(ip: string): 'private' | 'loopback' | 'public' {
		// IPv4 private ranges
		if (ip.startsWith('10.') || 
		    ip.startsWith('192.168.') ||
		    /^172\.(1[6-9]|2[0-9]|3[0-1])\./.test(ip)) {
			return 'private';
		}

		// Loopback
		if (ip.startsWith('127.') || ip === '::1') {
			return 'loopback';
		}

		return 'public';
	}

	/**
	 * Format location string from IP info
	 */
	static formatLocation(info: IPInfo): string {
		const parts = [];
		if (info.city) parts.push(info.city);
		if (info.region) parts.push(info.region);
		if (info.country) parts.push(info.country);
		return parts.join(', ') || 'Unknown';
	}

	/**
	 * Get flag emoji from country code
	 */
	static getCountryFlag(countryCode: string): string {
		if (!countryCode || countryCode.length !== 2) return '';
		const codePoints = countryCode
			.toUpperCase()
			.split('')
			.map(char => 127397 + char.charCodeAt(0));
		return String.fromCodePoint(...codePoints);
	}
}