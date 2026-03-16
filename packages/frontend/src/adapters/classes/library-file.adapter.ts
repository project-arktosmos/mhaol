import { AdapterClass } from '$adapters/classes/adapter.class';

export class LibraryFileAdapter extends AdapterClass {
	constructor() {
		super('library-file');
	}

	formatSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		const value = bytes / Math.pow(1024, i);
		return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	getMediaTypeBadgeClass(mediaType: string): string {
		const map: Record<string, string> = {
			video: 'badge-primary'
		};
		return map[mediaType] ?? 'badge-ghost';
	}

	getMediaTypeLabel(mediaType: string): string {
		const map: Record<string, string> = {
			video: 'Video'
		};
		return map[mediaType] ?? mediaType;
	}

	getCategoryBadgeClass(categoryId: string): string {
		const map: Record<string, string> = {
			tv: 'badge-info',
			movies: 'badge-warning',
			uncategorized: 'badge-ghost'
		};
		return map[categoryId] ?? 'badge-ghost';
	}

	getCategoryLabel(categoryId: string): string {
		const map: Record<string, string> = {
			tv: 'TV',
			movies: 'Movies',
			uncategorized: 'Uncategorized'
		};
		return map[categoryId] ?? categoryId;
	}
}

export const libraryFileAdapter = new LibraryFileAdapter();
