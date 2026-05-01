import { get, writable, type Writable } from 'svelte/store';
import { userIdentityService } from '$lib/user-identity.service';

/// Every addon known to the cloud — single source of truth for valid
/// `firkin.addon` values. Each addon represents a single content kind, so
/// there is no separate `type`/`kind` field on a firkin: the addon implies
/// it.
export const FIRKIN_ADDONS = [
	'tmdb-movie',
	'tmdb-tv',
	'musicbrainz',
	'retroachievements',
	'youtube-video',
	'youtube-channel',
	'wyzie-subs-movie',
	'wyzie-subs-tv',
	'lrclib',
	'local-movie',
	'local-tv',
	'local-album',
	'local-book',
	'local-game'
] as const;

export type FirkinAddon = (typeof FIRKIN_ADDONS)[number];

/// The display kind each addon produces. Used for UI labels and to drive
/// kind-aware behavior (audio playback, torrent search categories, etc.)
/// without re-introducing a stored `type` field on the firkin.
export const ADDON_KIND: Record<FirkinAddon, FirkinKind> = {
	'tmdb-movie': 'movie',
	'tmdb-tv': 'tv show',
	musicbrainz: 'album',
	retroachievements: 'game',
	'youtube-video': 'youtube video',
	'youtube-channel': 'youtube channel',
	'wyzie-subs-movie': 'movie',
	'wyzie-subs-tv': 'tv show',
	lrclib: 'album',
	'local-movie': 'movie',
	'local-tv': 'tv show',
	'local-album': 'album',
	'local-book': 'book',
	'local-game': 'game'
};

export const FIRKIN_KINDS = [
	'movie',
	'tv show',
	'album',
	'youtube video',
	'youtube channel',
	'book',
	'game'
] as const;

export type FirkinKind = (typeof FIRKIN_KINDS)[number];

export function addonKind(addon: string): FirkinKind | null {
	return (addon as FirkinAddon) in ADDON_KIND ? ADDON_KIND[addon as FirkinAddon] : null;
}

/// For a given firkin addon, the catalog/remote addon whose API can supply
/// metadata (poster, description, year, canonical title) for a match.
/// `local-*` firkins resolve to their remote counterpart; remote browsable
/// addons resolve to themselves so a manually-bookmarked TMDB entry can
/// still be re-searched. Anything not in the map (wyzie-subs, lrclib,
/// youtube-*, local-book) returns null and the metadata-search affordance
/// stays hidden.
const METADATA_SEARCH_ADDON: Record<string, string> = {
	'local-movie': 'tmdb-movie',
	'local-tv': 'tmdb-tv',
	'local-album': 'musicbrainz',
	'local-game': 'retroachievements',
	'tmdb-movie': 'tmdb-movie',
	'tmdb-tv': 'tmdb-tv',
	musicbrainz: 'musicbrainz',
	retroachievements: 'retroachievements'
};

export function metadataSearchAddon(addon: string): string | null {
	return METADATA_SEARCH_ADDON[addon] ?? null;
}

export interface Artist {
	/** CID of the persisted `artist` doc; absent on transient catalog/search items prior to bookmark. */
	id?: string;
	name: string;
	/** Single-occurrence role on inbound side (catalog/search → firkin upsert). */
	role?: string;
	/** Canonical multi-role array on resolved side (firkin GET / artist list). */
	roles?: string[];
	imageUrl?: string;
}

export interface ImageMeta {
	url: string;
	mimeType: string;
	fileSize: number;
	width: number;
	height: number;
}

export const FILE_TYPES = ['ipfs', 'torrent magnet', 'url'] as const;
export type FileType = (typeof FILE_TYPES)[number];

export interface FileEntry {
	type: FileType;
	value: string;
	title?: string;
}

export interface Firkin {
	id: string;
	title: string;
	/** CIDs of the referenced artist docs (cloud `artist` table), in order. */
	artistIds?: string[];
	/** Resolved artist bodies (server-side join from `artistIds`). */
	artists: Artist[];
	description: string;
	images: ImageMeta[];
	files: FileEntry[];
	year: number | null;
	addon: string;
	/** EVM address of the account that created the firkin. Empty for server-side auto-creates. */
	creator: string;
	created_at: string;
	updated_at: string;
	version?: number;
	version_hashes?: string[];
}

export interface FirkinsState {
	loading: boolean;
	firkins: Firkin[];
	error: string | null;
}

const initialState: FirkinsState = {
	loading: false,
	firkins: [],
	error: null
};

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

const POLL_INTERVAL_MS = 4000;

class FirkinsService {
	state: Writable<FirkinsState> = writable(initialState);

	private subscribers = 0;
	private timer: ReturnType<typeof setInterval> | null = null;
	private inFlight = false;

	start(): () => void {
		this.subscribers += 1;
		if (this.subscribers === 1) {
			void this.refresh();
			this.timer = setInterval(() => {
				void this.refresh();
			}, POLL_INTERVAL_MS);
		}
		return () => this.stop();
	}

	private stop(): void {
		this.subscribers = Math.max(0, this.subscribers - 1);
		if (this.subscribers === 0 && this.timer) {
			clearInterval(this.timer);
			this.timer = null;
		}
	}

	async refresh(): Promise<void> {
		if (this.inFlight) return;
		this.inFlight = true;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/firkins', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const firkins = (await res.json()) as Firkin[];
			this.state.set({ loading: false, firkins, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		} finally {
			this.inFlight = false;
		}
	}

	async create(input: {
		title: string;
		artists: Artist[];
		description: string;
		images: ImageMeta[];
		files: FileEntry[];
		year: number | null;
		addon: FirkinAddon;
		/** Override the auto-filled creator. Leave undefined to use the current user identity. */
		creator?: string;
	}): Promise<Firkin> {
		// Auto-fill the creator from the browser-resident user identity (the
		// same one the profile page manages). User identity is initialised on
		// app mount in +layout.svelte, so by the time any UI flow can call
		// create() the address is available — but we tolerate it being
		// missing and send an empty string so the request still succeeds.
		const identity = get(userIdentityService.state).identity;
		const payload = {
			...input,
			creator: input.creator ?? identity?.address ?? ''
		};
		const res = await fetch('/api/firkins', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(payload)
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Firkin;
		this.state.update((s) => {
			const existing = s.firkins.findIndex((d) => d.id === created.id);
			if (existing >= 0) {
				const next = s.firkins.slice();
				next[existing] = created;
				return { ...s, firkins: next };
			}
			return { ...s, firkins: [...s.firkins, created] };
		});
		return created;
	}

	/// Apply catalog-derived metadata to a firkin and roll its version
	/// forward. The server returns the new firkin (under a new CID). The
	/// caller is responsible for navigating to the new id, since the URL
	/// of the detail page is content-addressed.
	async enrich(
		id: string,
		payload: {
			title?: string;
			year?: number | null;
			description?: string;
			posterUrl?: string | null;
			backdropUrl?: string | null;
		}
	): Promise<Firkin> {
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}/enrich`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(payload)
		});
		if (!res.ok) throw new Error(await parseError(res));
		const updated = (await res.json()) as Firkin;
		this.state.update((s) => {
			const next = s.firkins.filter((d) => d.id !== id && d.id !== updated.id);
			next.push(updated);
			return { ...s, firkins: next };
		});
		return updated;
	}

	async remove(id: string): Promise<void> {
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!res.ok && res.status !== 204) throw new Error(await parseError(res));
		this.state.update((s) => ({
			...s,
			firkins: s.firkins.filter((d) => d.id !== id)
		}));
	}
}

export const firkinsService = new FirkinsService();
