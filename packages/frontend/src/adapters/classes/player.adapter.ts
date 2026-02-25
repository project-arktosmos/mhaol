import { AdapterClass } from '$adapters/classes/adapter.class';
import type { PlayableFile } from '$types/player.type';

export class PlayerAdapter extends AdapterClass {
	constructor() {
		super('player');
	}

	formatDuration(seconds: number | null): string {
		if (seconds === null || seconds < 0) return '--';
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = Math.floor(seconds % 60);
		if (h > 0) {
			return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
		}
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	formatSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		const value = bytes / Math.pow(1024, i);
		return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	getFormatLabel(file: PlayableFile): string {
		if (file.videoFormat) return file.videoFormat.toUpperCase();
		if (file.format) return file.format.toUpperCase();
		return 'Unknown';
	}

	getSourceBadgeClass(type: 'youtube' | 'torrent'): string {
		return type === 'youtube' ? 'badge-secondary' : 'badge-accent';
	}
}

export const playerAdapter = new PlayerAdapter();
