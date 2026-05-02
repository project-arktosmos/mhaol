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
export const FILE_TYPES = ['ipfs', 'torrent magnet', 'url', 'lyrics', 'subtitle'] as const;

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
const INCLUDE_ALL_STORAGE_KEY = 'mhaol-firkins-include-all';

function readStoredIncludeAll(): boolean {
	if (typeof localStorage === 'undefined') return false;
	try {
		return localStorage.getItem(INCLUDE_ALL_STORAGE_KEY) === '1';
	} catch {
		return false;
	}
}

function writeStoredIncludeAll(value: boolean): void {
	if (typeof localStorage === 'undefined') return;
	try {
		localStorage.setItem(INCLUDE_ALL_STORAGE_KEY, value ? '1' : '0');
	} catch {
		// ignore — localStorage might be disabled
	}
}

class FirkinsService {
	state: Writable<FirkinsState> = writable(initialState);
	/// Public store so any consumer can react to mode flips. The catalog
	/// page renders a toggle bound to this store; the library section
	/// re-renders when the toggle flips because `state.firkins` is
	/// re-fetched against the new mode.
	includeAll: Writable<boolean> = writable(readStoredIncludeAll());

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

	/// Toggle between bookmarked-only (`false`, default) and every
	/// firkin in the local DB (`true`, including non-bookmarked
	/// browse-cache rows from the `/catalog/visit` resolver). Persists
	/// to localStorage so the choice sticks across reloads, and triggers
	/// an immediate refresh so the store reflects the new mode.
	setIncludeAll(value: boolean): void {
		const current = get(this.includeAll);
		if (current === value) return;
		this.includeAll.set(value);
		writeStoredIncludeAll(value);
		void this.refresh();
	}

	async refresh(): Promise<void> {
		if (this.inFlight) return;
		this.inFlight = true;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const url = get(this.includeAll) ? '/api/firkins?include=all' : '/api/firkins';
			const res = await fetch(url, { cache: 'no-store' });
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
		/**
		 * Mark the firkin as bookmarked. Defaults to `true` server-side when
		 * the field is omitted, so the catalog "Bookmark" button doesn't
		 * need to set it. The `/catalog/visit` resolver flow passes `false`
		 * to register a non-bookmarked browse-cache firkin.
		 */
		bookmarked?: boolean;
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
		// Keep local state in sync with what `GET /api/firkins` would return
		// for the current toggle. Non-bookmarked browse-cache firkins from
		// the `/catalog/visit` resolver (created by SvelteKit prefetch on
		// hover) must NOT appear in bookmarked-only mode — otherwise the
		// catalog Library row briefly gains items the user only hovered.
		const includeAllNow = get(this.includeAll);
		const visibleInCurrentMode = includeAllNow || created.bookmarked !== false;
		this.state.update((s) => {
			const existing = s.firkins.findIndex((d) => d.id === created.id);
			if (existing >= 0) {
				if (!visibleInCurrentMode) {
					const next = s.firkins.slice();
					next.splice(existing, 1);
					return { ...s, firkins: next };
				}
				const next = s.firkins.slice();
				next[existing] = created;
				return { ...s, firkins: next };
			}
			if (!visibleInCurrentMode) return s;
			return { ...s, firkins: [...s.firkins, created] };
		});
		return created;
	}

	/// Apply catalog-derived metadata to a firkin in place. The server
	/// recomputes the body CID and bumps the version, but the record id
	/// (UUID) stays the same — the response is just the updated firkin
	/// at the same id.
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
	/// update the firkin in place (recomputed body CID, bumped version).
	/// Only valid for `musicbrainz` firkins.
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

	/// Promote a non-bookmarked browse-cache firkin (created by the
	/// catalog `/catalog/visit` resolver) to a bookmarked one. The server
	/// flips the flag in place — `bookmarked` is not part of the firkin
	/// body, so the CID and version are unchanged. Returns the updated
	/// firkin so the caller can refresh its local copy.
	async bookmark(id: string): Promise<Firkin> {
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}`, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ bookmarked: true })
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

	/// Download a picked subtitle, pin it to IPFS, and attach it to the
	/// firkin as a `subtitle`-typed FileEntry. The backend converts SRT
	/// to VTT before pinning so the in-page player's parser can read it
	/// directly from `/api/ipfs/pins/<cid>/file`. Returns the rolled-
	/// forward firkin so the caller can refresh local state.
	async attachSubtitle(
		id: string,
		payload: {
			source: string;
			externalId: string;
			url: string;
			language: string;
			display?: string | null;
			release?: string | null;
			format?: string | null;
			isHearingImpaired?: boolean;
			/** TV-only — identifies which episode the subtitle is timed to. */
			season?: number;
			episode?: number;
		}
	): Promise<Firkin> {
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}/subtitle`, {
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

	/// Granular files mutation. Always prefer this over a `PUT` with a
	/// full `files` array — the server reads the current state under the
	/// per-firkin async lock, removes matching entries, appends the new
	/// ones, and rolls forward, so two concurrent callers never lose
	/// each other's writes. The legacy `PUT /api/firkins/:id` with a
	/// `files` field replaces wholesale from a *client* snapshot, which
	/// is racy: the catalog detail page used to drop a freshly-attached
	/// `torrent magnet` whenever the trailer resolver's
	/// `youtube preferred client` write landed in the same window.
	async mutateFiles(
		id: string,
		patch: {
			add?: FileEntry[];
			removeTypes?: FileType[];
			removeEntries?: { type: FileType; value: string }[];
		}
	): Promise<Firkin> {
		const body = {
			add: patch.add ?? [],
			removeTypes: patch.removeTypes ?? [],
			removeEntries: patch.removeEntries ?? []
		};
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}/files`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body)
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
}

export const firkinsService = new FirkinsService();
