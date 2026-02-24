import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { PlayableFile } from '$types/player.type';

export const GET: RequestHandler = async ({ locals }) => {
	const youtubeRows = locals.youtubeDownloadRepo.getByState('completed');
	const torrentRows = locals.torrentDownloadRepo
		.getAll()
		.filter((r) => r.state === 'seeding' || r.progress >= 1);

	const playable: PlayableFile[] = [];

	for (const row of youtubeRows) {
		if (!row.output_path) continue;
		playable.push({
			id: row.download_id,
			type: 'youtube',
			name: row.title,
			outputPath: row.output_path,
			mode: row.mode as 'audio' | 'video',
			format: row.format,
			videoFormat: row.video_format,
			thumbnailUrl: row.thumbnail_url,
			durationSeconds: row.duration_seconds,
			size: row.total_bytes,
			completedAt: row.updated_at
		});
	}

	for (const row of torrentRows) {
		if (!row.output_path) continue;
		playable.push({
			id: row.info_hash,
			type: 'torrent',
			name: row.name,
			outputPath: row.output_path && row.name ? `${row.output_path}/${row.name}` : row.output_path,
			mode: 'video',
			format: null,
			videoFormat: null,
			thumbnailUrl: null,
			durationSeconds: null,
			size: row.size,
			completedAt: row.updated_at
		});
	}

	playable.sort((a, b) => new Date(b.completedAt).getTime() - new Date(a.completedAt).getTime());

	return json(playable);
};
