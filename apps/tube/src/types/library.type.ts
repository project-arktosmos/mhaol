import type { ID } from '$types/core.type';

export interface LibraryCardItem {
	videoId: string;
	title: string;
	thumbnailUrl: string | null;
	durationSeconds: number | null;
	channelName: string | null;
	hasVideo: boolean;
	hasAudio: boolean;
}

export interface Library {
	id: ID;
	name: string;
	path: string;
	dateAdded: number;
}

export interface LibraryFsEntry {
	name: string;
	size: number;
}

export interface LibraryFs {
	path: string;
	audio: LibraryFsEntry[];
	video: LibraryFsEntry[];
}
