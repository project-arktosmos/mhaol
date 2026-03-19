import { AdapterClass } from 'frontend/adapters/classes/adapter.class';

class CloudAdapter extends AdapterClass {
	constructor() {
		super('cloud');
	}

	formatBytes(bytes: number | null): string {
		if (bytes === null || bytes === 0) return '0 B';

		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		const value = bytes / Math.pow(1024, i);
		return `${value.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
	}

	extensionBadgeClass(extension: string): string {
		const videoExts = ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'm4v'];
		const audioExts = ['mp3', 'flac', 'wav', 'ogg', 'm4a', 'opus', 'aac'];
		const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'];

		if (videoExts.includes(extension)) return 'badge-primary';
		if (audioExts.includes(extension)) return 'badge-secondary';
		if (imageExts.includes(extension)) return 'badge-accent';
		return 'badge-ghost';
	}

	attributeTypeBadgeClass(typeId: string): string {
		switch (typeId) {
			case 'string':
				return 'badge-info';
			case 'number':
				return 'badge-success';
			case 'boolean':
				return 'badge-warning';
			case 'date':
				return 'badge-primary';
			case 'url':
				return 'badge-secondary';
			case 'duration':
				return 'badge-accent';
			case 'bytes':
				return 'badge-neutral';
			case 'tags':
				return 'badge-error';
			case 'json':
				return 'badge-ghost';
			default:
				return 'badge-ghost';
		}
	}

	formatAttributeValue(value: string, typeId: string): string {
		switch (typeId) {
			case 'bytes': {
				const bytes = parseInt(value, 10);
				return isNaN(bytes) ? value : this.formatBytes(bytes);
			}
			case 'duration': {
				const seconds = parseInt(value, 10);
				if (isNaN(seconds)) return value;
				const m = Math.floor(seconds / 60);
				const s = seconds % 60;
				return `${m}:${s.toString().padStart(2, '0')}`;
			}
			case 'boolean':
				return value === 'true' || value === '1' ? 'Yes' : 'No';
			default:
				return value;
		}
	}
}

export const cloudAdapter = new CloudAdapter();
