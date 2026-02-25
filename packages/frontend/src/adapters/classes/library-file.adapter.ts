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
			[MediaType.Images]: 'badge-secondary',
			[MediaType.Music]: 'badge-accent'
		};
		return map[mediaType];
	}

	getMediaTypeLabel(mediaType: MediaType): string {
		const map: Record<MediaType, string> = {
			[MediaType.Video]: 'Video',
			[MediaType.Images]: 'Image',
			[MediaType.Music]: 'Music'
		};
		return map[mediaType];
	}
}

export const libraryFileAdapter = new LibraryFileAdapter();
