import { AdapterClass } from '$adapters/classes/adapter.class';
import { MediaType } from '$types/library.type';

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

	getMediaTypeBadgeClass(mediaType: MediaType): string {
		const map: Record<MediaType, string> = {
			[MediaType.Video]: 'badge-primary',
			[MediaType.Image]: 'badge-secondary',
			[MediaType.Audio]: 'badge-accent'
		};
		return map[mediaType];
	}

	getMediaTypeLabel(mediaType: MediaType): string {
		const map: Record<MediaType, string> = {
			[MediaType.Video]: 'Video',
			[MediaType.Image]: 'Image',
			[MediaType.Audio]: 'Audio'
		};
		return map[mediaType];
	}

	getCategoryBadgeClass(categoryId: string): string {
		const map: Record<string, string> = {
			tv: 'badge-info',
			movies: 'badge-warning',
			youtube: 'badge-error',
			uncategorized: 'badge-ghost'
		};
		return map[categoryId] ?? 'badge-ghost';
	}

	getCategoryLabel(categoryId: string): string {
		const map: Record<string, string> = {
			tv: 'TV',
			movies: 'Movies',
			youtube: 'YouTube',
			uncategorized: 'Uncategorized'
		};
		return map[categoryId] ?? categoryId;
	}
}

export const libraryFileAdapter = new LibraryFileAdapter();
