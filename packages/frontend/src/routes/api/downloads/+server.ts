import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { UnifiedDownload } from '$types/download.type';

export const GET: RequestHandler = async ({ locals }) => {
	const youtubeRows = locals.youtubeDownloadRepo.getAll();
	const torrentRows = locals.torrentDownloadRepo.getAll();

	const youtubeDownloads: UnifiedDownload[] = youtubeRows.map((row) => ({
		id: row.download_id,
		type: 'youtube',
		name: row.title,
		state: row.state,
		progress: row.progress,
		size: row.total_bytes,
		outputPath: row.output_path,
		error: row.error,
		createdAt: row.created_at,
		updatedAt: row.updated_at,
		url: row.url,
		mode: row.mode,
		format: row.format
	}));

	const torrentDownloads: UnifiedDownload[] = torrentRows.map((row) => ({
		id: row.info_hash,
		type: 'torrent',
		name: row.name,
		state: row.state,
		progress: row.progress,
		size: row.size,
		outputPath: row.output_path && row.name ? `${row.output_path}/${row.name}` : row.output_path,
		error: null,
		createdAt: row.created_at,
		updatedAt: row.updated_at,
		downloadSpeed: row.download_speed,
		uploadSpeed: row.upload_speed,
		peers: row.peers,
		seeds: row.seeds,
		eta: row.eta
	}));

	const downloads = [...youtubeDownloads, ...torrentDownloads].sort(
		(a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
	);

	return json(downloads);
};
