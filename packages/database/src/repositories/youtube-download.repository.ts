import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { YouTubeDownloadRow } from '../types.js';

export class YouTubeDownloadRepository {
	private stmts: {
		get: Statement<[string], YouTubeDownloadRow>;
		getAll: Statement<[], YouTubeDownloadRow>;
		getByState: Statement<[string], YouTubeDownloadRow>;
		upsert: Statement<
			[{
				download_id: string;
				url: string;
				video_id: string;
				title: string;
				state: string;
				progress: number;
				downloaded_bytes: number;
				total_bytes: number;
				output_path: string | null;
				error: string | null;
				mode: string;
				quality: string;
				format: string;
				video_quality: string | null;
				video_format: string | null;
				thumbnail_url: string | null;
				duration_seconds: number | null;
			}]
		>;
		updateState: Statement<
			[{
				download_id: string;
				state: string;
				progress: number;
				output_path: string | null;
				error: string | null;
			}]
		>;
		delete: Statement<[string]>;
		deleteByState: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM youtube_downloads WHERE download_id = ?'),
			getAll: db.prepare('SELECT * FROM youtube_downloads ORDER BY created_at DESC'),
			getByState: db.prepare(
				'SELECT * FROM youtube_downloads WHERE state = ? ORDER BY created_at DESC'
			),
			upsert: db.prepare(`
				INSERT INTO youtube_downloads (
					download_id, url, video_id, title, state, progress,
					downloaded_bytes, total_bytes, output_path, error,
					mode, quality, format, video_quality, video_format,
					thumbnail_url, duration_seconds
				) VALUES (
					@download_id, @url, @video_id, @title, @state, @progress,
					@downloaded_bytes, @total_bytes, @output_path, @error,
					@mode, @quality, @format, @video_quality, @video_format,
					@thumbnail_url, @duration_seconds
				)
				ON CONFLICT(download_id) DO UPDATE SET
					title = @title, state = @state, progress = @progress,
					downloaded_bytes = @downloaded_bytes, total_bytes = @total_bytes,
					output_path = @output_path, error = @error,
					video_quality = @video_quality, video_format = @video_format,
					thumbnail_url = @thumbnail_url, duration_seconds = @duration_seconds
			`),
			updateState: db.prepare(`
				UPDATE youtube_downloads SET
					state = @state, progress = @progress,
					output_path = @output_path, error = @error
				WHERE download_id = @download_id
			`),
			delete: db.prepare('DELETE FROM youtube_downloads WHERE download_id = ?'),
			deleteByState: db.prepare('DELETE FROM youtube_downloads WHERE state = ?')
		};
	}

	get(downloadId: string): YouTubeDownloadRow | null {
		return this.stmts.get.get(downloadId) ?? null;
	}

	getAll(): YouTubeDownloadRow[] {
		return this.stmts.getAll.all();
	}

	getByState(state: string): YouTubeDownloadRow[] {
		return this.stmts.getByState.all(state);
	}

	upsert(row: Omit<YouTubeDownloadRow, 'created_at' | 'updated_at'>): void {
		this.stmts.upsert.run(row);
	}

	updateState(
		downloadId: string,
		updates: { state: string; progress: number; outputPath: string | null; error: string | null }
	): void {
		this.stmts.updateState.run({
			download_id: downloadId,
			state: updates.state,
			progress: updates.progress,
			output_path: updates.outputPath,
			error: updates.error
		});
	}

	delete(downloadId: string): boolean {
		const result = this.stmts.delete.run(downloadId);
		return result.changes > 0;
	}

	deleteByStates(states: string[]): void {
		const transaction = this.db.transaction((stateList: string[]) => {
			for (const state of stateList) {
				this.stmts.deleteByState.run(state);
			}
		});
		transaction(states);
	}
}
