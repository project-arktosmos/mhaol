import type { Database as DatabaseType, Statement } from 'better-sqlite3';

export interface CacheRow {
	service: string;
	resource_type: string;
	cache_key: string;
	data: string;
	fetched_at: string;
}

export interface ImageCacheRow {
	service: string;
	image_key: string;
	url: string;
	local_path: string;
	mime_type: string | null;
	cached_at: string;
}

export const ADDON_CACHE_SCHEMA = `
CREATE TABLE IF NOT EXISTS addon_cache (
    service TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    cache_key TEXT NOT NULL,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (service, resource_type, cache_key)
);

CREATE TABLE IF NOT EXISTS addon_image_cache (
    service TEXT NOT NULL,
    image_key TEXT NOT NULL,
    url TEXT NOT NULL,
    local_path TEXT NOT NULL,
    mime_type TEXT,
    cached_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (service, image_key)
);
`;

const DEFAULT_STALE_DAYS = 7;

export class AddonCacheRepository {
	private stmts: {
		get: Statement<[string, string, string], CacheRow>;
		upsert: Statement<[string, string, string, string]>;
		delete: Statement<[string, string, string]>;
		deleteByService: Statement<[string]>;
		deleteByServiceType: Statement<[string, string]>;
		getImage: Statement<[string, string], ImageCacheRow>;
		upsertImage: Statement<[string, string, string, string, string | null]>;
		deleteImage: Statement<[string, string]>;
		deleteImagesByService: Statement<[string]>;
	};

	constructor(
		private db: DatabaseType,
		private staleDays: number = DEFAULT_STALE_DAYS
	) {
		this.stmts = {
			get: db.prepare(
				'SELECT * FROM addon_cache WHERE service = ? AND resource_type = ? AND cache_key = ?'
			),
			upsert: db.prepare(`
				INSERT INTO addon_cache (service, resource_type, cache_key, data) VALUES (?, ?, ?, ?)
				ON CONFLICT(service, resource_type, cache_key) DO UPDATE SET data = excluded.data, fetched_at = datetime('now')
			`),
			delete: db.prepare(
				'DELETE FROM addon_cache WHERE service = ? AND resource_type = ? AND cache_key = ?'
			),
			deleteByService: db.prepare('DELETE FROM addon_cache WHERE service = ?'),
			deleteByServiceType: db.prepare(
				'DELETE FROM addon_cache WHERE service = ? AND resource_type = ?'
			),
			getImage: db.prepare('SELECT * FROM addon_image_cache WHERE service = ? AND image_key = ?'),
			upsertImage: db.prepare(`
				INSERT INTO addon_image_cache (service, image_key, url, local_path, mime_type) VALUES (?, ?, ?, ?, ?)
				ON CONFLICT(service, image_key) DO UPDATE SET url = excluded.url, local_path = excluded.local_path, mime_type = excluded.mime_type, cached_at = datetime('now')
			`),
			deleteImage: db.prepare('DELETE FROM addon_image_cache WHERE service = ? AND image_key = ?'),
			deleteImagesByService: db.prepare('DELETE FROM addon_image_cache WHERE service = ?')
		};
	}

	// --- Data cache ---

	get(service: string, resourceType: string, cacheKey: string): CacheRow | null {
		return this.stmts.get.get(service, resourceType, cacheKey) ?? null;
	}

	getFresh(service: string, resourceType: string, cacheKey: string): CacheRow | null {
		const row = this.get(service, resourceType, cacheKey);
		if (row && this.isFresh(row.fetched_at)) return row;
		return null;
	}

	upsert(service: string, resourceType: string, cacheKey: string, data: string): void {
		this.stmts.upsert.run(service, resourceType, cacheKey, data);
	}

	delete(service: string, resourceType: string, cacheKey: string): boolean {
		return this.stmts.delete.run(service, resourceType, cacheKey).changes > 0;
	}

	deleteByService(service: string): number {
		return this.stmts.deleteByService.run(service).changes;
	}

	deleteByServiceType(service: string, resourceType: string): number {
		return this.stmts.deleteByServiceType.run(service, resourceType).changes;
	}

	// --- Image cache ---

	getImage(service: string, imageKey: string): ImageCacheRow | null {
		return this.stmts.getImage.get(service, imageKey) ?? null;
	}

	upsertImage(
		service: string,
		imageKey: string,
		url: string,
		localPath: string,
		mimeType: string | null = null
	): void {
		this.stmts.upsertImage.run(service, imageKey, url, localPath, mimeType);
	}

	deleteImage(service: string, imageKey: string): boolean {
		return this.stmts.deleteImage.run(service, imageKey).changes > 0;
	}

	deleteImagesByService(service: string): number {
		return this.stmts.deleteImagesByService.run(service).changes;
	}

	// --- Freshness ---

	isFresh(fetchedAt: string, staleDaysOverride?: number): boolean {
		const fetched = new Date(fetchedAt + 'Z');
		const now = new Date();
		const diffMs = now.getTime() - fetched.getTime();
		const diffDays = diffMs / (1000 * 60 * 60 * 24);
		return diffDays < (staleDaysOverride ?? this.staleDays);
	}
}
