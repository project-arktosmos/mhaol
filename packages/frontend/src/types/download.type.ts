export interface UnifiedDownload {
	id: string;
	type: 'torrent' | 'youtube-video' | 'youtube-audio';
	name: string;
	state: string;
	progress: number;
	size: number;
	outputPath: string | null;
	error: string | null;
	createdAt: string;
	updatedAt: string;
	// YouTube-specific
	url?: string;
	videoId?: string;
	thumbnailUrl?: string | null;
	durationSeconds?: number | null;
	// Torrent-specific
	downloadSpeed?: number;
	uploadSpeed?: number;
	peers?: number;
	seeds?: number;
	eta?: number | null;
}
