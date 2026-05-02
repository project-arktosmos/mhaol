import { get, writable, type Writable } from 'svelte/store';
import { userIdentityService } from '$lib/user-identity.service';
import {
	FIRKIN_ADDONS,
	FIRKIN_KINDS,
	ADDON_KIND,
	addonKind,
	type FirkinAddon,
	type FirkinKind,
	type Firkin as SharedFirkin,
	type FirkinArtist as SharedArtist,
	type FirkinImage as SharedImageMeta,
	type FirkinFile as SharedFileEntry,
	type FirkinFileType as SharedFileType,
	type FirkinTrailer as SharedTrailer,
	type FirkinReview as SharedReview
} from 'cloud-ui';

export { FIRKIN_ADDONS, FIRKIN_KINDS, ADDON_KIND, addonKind, type FirkinAddon, type FirkinKind };

/// For a given firkin addon, the catalog/remote addon whose API can supply
/// metadata (poster, description, year, canonical title) for a match.
const METADATA_SEARCH_ADDON: Record<string, string> = {
	'local-movie': 'tmdb-movie',
	'local-tv': 'tmdb-tv',
	'local-album': 'musicbrainz',
	'tmdb-movie': 'tmdb-movie',
	'tmdb-tv': 'tmdb-tv',
	musicbrainz: 'musicbrainz'
};

export function metadataSearchAddon(addon: string): string | null {
	return METADATA_SEARCH_ADDON[addon] ?? null;
}

export type Artist = SharedArtist;
export type ImageMeta = SharedImageMeta;
export type FileEntry = SharedFileEntry;
export type FileType = SharedFileType;
export type Trailer = SharedTrailer;
export type Review = SharedReview;
export type Firkin = SharedFirkin;
export const FILE_TYPES = ['ipfs', 'torrent magnet', 'url', 'lyrics'] as const;

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
		/** Optional trailers to bake into the firkin at create time. */
		trailers?: Trailer[];
		/** Optional reviews (upstream user-ratings) to bake into the firkin at create time. */
		reviews?: Review[];
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

	/// Run the server-side track resolver: fetch the album's tracks from
	/// MusicBrainz, search YouTube + LRCLIB per track, pick the best match
	/// for each, pack the resulting URL / lyrics entries into `files`, and
	/// roll the firkin forward to a new content-addressed id (which the
	/// caller must navigate to). Only valid for `musicbrainz` firkins.
	async resolveTracks(id: string): Promise<Firkin> {
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}/resolve-tracks`, {
			method: 'POST'
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
