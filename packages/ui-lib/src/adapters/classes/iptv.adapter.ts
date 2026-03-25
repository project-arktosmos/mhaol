import { AdapterClass } from './adapter.class';
import type { IptvChannel } from 'ui-lib/types/iptv.type';

class IptvAdapter extends AdapterClass {
	constructor() {
		super('iptv');
	}

	getCountryFlag(code: string): string {
		if (!code || code.length !== 2) return '';
		const offset = 0x1f1e6;
		const a = code.toUpperCase().charCodeAt(0) - 65 + offset;
		const b = code.toUpperCase().charCodeAt(1) - 65 + offset;
		return String.fromCodePoint(a, b);
	}

	getCategoryBadgeClass(category: string): string {
		const map: Record<string, string> = {
			news: 'badge-info',
			sports: 'badge-success',
			entertainment: 'badge-primary',
			music: 'badge-secondary',
			movies: 'badge-accent',
			kids: 'badge-warning',
			education: 'badge-neutral'
		};
		return map[category.toLowerCase()] ?? 'badge-ghost';
	}

	formatChannelSubtitle(channel: IptvChannel): string {
		const parts: string[] = [];
		if (channel.country) {
			const flag = this.getCountryFlag(channel.country);
			parts.push(flag ? `${flag} ${channel.country}` : channel.country);
		}
		if (channel.categories.length > 0) {
			parts.push(channel.categories.join(', '));
		}
		return parts.join(' · ');
	}

	getStreamUrl(channelId: string): string {
		return `/api/iptv/stream/${encodeURIComponent(channelId)}`;
	}
}

export const iptvAdapter = new IptvAdapter();
