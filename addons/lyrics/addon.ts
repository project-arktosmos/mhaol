import type { Database as DatabaseType } from 'better-sqlite3';
import type { PluginCompanion } from '../../packages/frontend/src/lib/server/plugins/types';
import { LrcLibCacheRepository } from './src/cache-repository';

interface LegacyLyricsCacheRow {
	library_item_id: string;
	data: string;
	fetched_at: string;
}

export const lyricsCompanion: PluginCompanion = {
	repositories: [{ class: LrcLibCacheRepository, localsKey: 'lrclibCacheRepo' }],
	schema: {
		migrations: (db: DatabaseType) => {
			const oldTable = db
				.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='lyrics_cache'")
				.get() as { name: string } | undefined;
			if (!oldTable) return;

			const rows = db.prepare('SELECT * FROM lyrics_cache').all() as LegacyLyricsCacheRow[];
			for (const row of rows) {
				const data = JSON.parse(row.data);
				if (data.id && data.id !== 0) {
					db.prepare(
						`INSERT OR IGNORE INTO lrclib_lyrics (lrclib_id, track_name, artist_name, album_name, duration, instrumental, plain_lyrics, synced_lyrics)
						 VALUES (?, ?, ?, ?, ?, ?, ?, NULL)`
					).run(
						data.id,
						data.trackName,
						data.artistName,
						data.albumName,
						data.duration,
						data.instrumental ? 1 : 0,
						data.plainLyrics
					);
					db.prepare(
						`INSERT OR IGNORE INTO lrclib_lookups (library_item_id, lrclib_id, status) VALUES (?, ?, 'found')`
					).run(row.library_item_id, data.id);
				} else {
					db.prepare(
						`INSERT OR IGNORE INTO lrclib_lookups (library_item_id, lrclib_id, status) VALUES (?, NULL, 'not_found')`
					).run(row.library_item_id);
				}
			}

			db.exec('DROP TABLE lyrics_cache');
		}
	}
};
