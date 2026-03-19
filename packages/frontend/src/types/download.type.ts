export interface UnifiedDownload {
	id: string;
	type: 'youtube' | 'torrent';
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
	mode?: string;
	format?: string;
	thumbnailUrl?: string | null;
	// Torrent-specific
	downloadSpeed?: number;
	uploadSpeed?: number;
	peers?: number;
	seeds?: number;
	eta?: number | null;
}
