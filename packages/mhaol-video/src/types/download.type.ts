export interface UnifiedDownload {
	id: string;
	type: 'torrent';
	name: string;
	state: string;
	progress: number;
	size: number;
	outputPath: string | null;
	error: string | null;
	createdAt: string;
	updatedAt: string;
	downloadSpeed?: number;
	uploadSpeed?: number;
	peers?: number;
	seeds?: number;
	eta?: number | null;
}
