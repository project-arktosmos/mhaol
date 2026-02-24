import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { TorrentDownloadRow } from '../types.js';

export class TorrentDownloadRepository {
	private stmts: {
		get: Statement<[string], TorrentDownloadRow>;
		getAll: Statement<[], TorrentDownloadRow>;
		upsert: Statement<
			[{
				info_hash: string;
				name: string;
				size: number;
				progress: number;
				state: string;
				download_speed: number;
				upload_speed: number;
				peers: number;
				seeds: number;
				added_at: number;
				eta: number | null;
				output_path: string | null;
				source: string;
			}]
		>;
		updateState: Statement<
			[{
				info_hash: string;
				progress: number;
				state: string;
				download_speed: number;
				upload_speed: number;
				peers: number;
				seeds: number;
				eta: number | null;
				output_path: string | null;
			}]
		>;
		delete: Statement<[string]>;
		deleteAll: Statement<[]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM torrent_downloads WHERE info_hash = ?'),
			getAll: db.prepare('SELECT * FROM torrent_downloads ORDER BY added_at DESC'),
			upsert: db.prepare(`
				INSERT INTO torrent_downloads (
					info_hash, name, size, progress, state,
					download_speed, upload_speed, peers, seeds,
					added_at, eta, output_path, source
				) VALUES (
					@info_hash, @name, @size, @progress, @state,
					@download_speed, @upload_speed, @peers, @seeds,
					@added_at, @eta, @output_path, @source
				)
				ON CONFLICT(info_hash) DO UPDATE SET
					name = @name, size = @size, progress = @progress, state = @state,
					download_speed = @download_speed, upload_speed = @upload_speed,
					peers = @peers, seeds = @seeds,
					eta = @eta, output_path = @output_path
			`),
			updateState: db.prepare(`
				UPDATE torrent_downloads SET
					progress = @progress, state = @state,
					download_speed = @download_speed, upload_speed = @upload_speed,
					peers = @peers, seeds = @seeds,
					eta = @eta, output_path = @output_path
				WHERE info_hash = @info_hash
			`),
			delete: db.prepare('DELETE FROM torrent_downloads WHERE info_hash = ?'),
			deleteAll: db.prepare('DELETE FROM torrent_downloads')
		};
	}

	get(infoHash: string): TorrentDownloadRow | null {
		return this.stmts.get.get(infoHash) ?? null;
	}

	getAll(): TorrentDownloadRow[] {
		return this.stmts.getAll.all();
	}

	upsert(row: Omit<TorrentDownloadRow, 'created_at' | 'updated_at'>): void {
		this.stmts.upsert.run(row);
	}

	updateState(
		infoHash: string,
		updates: {
			progress: number;
			state: string;
			downloadSpeed: number;
			uploadSpeed: number;
			peers: number;
			seeds: number;
			eta: number | null;
			outputPath: string | null;
		}
	): void {
		this.stmts.updateState.run({
			info_hash: infoHash,
			progress: updates.progress,
			state: updates.state,
			download_speed: updates.downloadSpeed,
			upload_speed: updates.uploadSpeed,
			peers: updates.peers,
			seeds: updates.seeds,
			eta: updates.eta,
			output_path: updates.outputPath
		});
	}

	delete(infoHash: string): boolean {
		const result = this.stmts.delete.run(infoHash);
		return result.changes > 0;
	}

	deleteAll(): void {
		this.stmts.deleteAll.run();
	}
}
