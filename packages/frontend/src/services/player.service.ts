import { get, writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import localStorageWritableStore from '$utils/localStorageWritableStore';
import type {
	PlayerSettings,
	PlayerSnapshot,
	PlayerState,
	PlayerDisplayMode,
	PlayableFile,
	PlayerPlaylist
} from '$types/player.type';
import type { SubsLyricsSyncedLine } from '$types/subs-lyrics.type';

const initialSettings: PlayerSettings = {
	id: 'player-settings',
	preferredVolume: 1.0,
	autoplay: false
};

const initialState: PlayerState = {
	initialized: false,
	loading: false,
	error: null,
	files: [],
	currentFile: null,
	connectionState: 'idle',
	positionSecs: 0,
	durationSecs: null,
	isSeeking: false,
	isPaused: true,
	buffering: false,
	directStreamUrl: null,
	directStreamMimeType: null,
	streamOffsetSecs: 0,
	firkinId: null,
	trackId: null,
	trackTitle: null,
	syncedLyrics: null,
	pendingSeekSecs: null
};

const initialSnapshot: PlayerSnapshot = {
	displayMode: 'fullscreen',
	currentFile: null,
	positionSecs: 0,
	directStreamUrl: null,
	directStreamMimeType: null,
	firkinId: null,
	trackId: null,
	trackTitle: null,
	syncedLyrics: null,
	playlist: null
};

class PlayerService extends ObjectServiceClass<PlayerSettings> {
	public state: Writable<PlayerState> = writable(initialState);
	public displayMode: Writable<PlayerDisplayMode> = writable('fullscreen');
	public playlist: Writable<PlayerPlaylist | null> = writable(null);
	// Restorable view of the navbar player. Read once at boot to rehydrate
	// the panel after a refresh; written (throttled) by `startPersistingSnapshot`.
	public snapshot: Writable<PlayerSnapshot> = browser
		? localStorageWritableStore<PlayerSnapshot>('player-snapshot', initialSnapshot)
		: writable(initialSnapshot);

	setDisplayMode(mode: PlayerDisplayMode): void {
		this.displayMode.set(mode);
	}

	// ===== Playlist =====
	//
	// Track queue surfaced by the floating player panel. Set by callers
	// that want to expose a swap-list (e.g. the catalog tracks card),
	// preserved through every `beginLoad` / `playUrl` cycle so swapping
	// to another track doesn't wipe the queue, and only cleared on an
	// explicit user-driven `stop()`.

	setPlaylist(playlist: PlayerPlaylist | null): void {
		this.playlist.set(playlist);
	}

	setPlaylistIndex(index: number): void {
		this.playlist.update((p) => {
			if (!p) return p;
			if (index < 0 || index >= p.tracks.length) return p;
			if (p.currentIndex === index) return p;
			return { ...p, currentIndex: index };
		});
	}

	clearPlaylist(): void {
		this.playlist.set(null);
	}

	private _initialized = false;
	private seekTimeout: ReturnType<typeof setTimeout> | null = null;
	private playGeneration = 0;

	constructor() {
		super('player-settings', initialSettings);
	}

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this.state.update((s) => ({ ...s, initialized: true, loading: false, error: null }));
		this._initialized = true;
		this._startPersistingSnapshot();
	}

	// ===== Snapshot persistence =====
	//
	// Throttle snapshot writes so the ~4Hz `timeupdate` cadence doesn't hammer
	// localStorage with full-state JSON. Cleared on `stop()`; otherwise updated
	// on every state / playlist / displayMode change while the navbar player is
	// active.
	private _persistingStarted = false;
	private _persistTimer: ReturnType<typeof setTimeout> | null = null;

	private _startPersistingSnapshot(): void {
		if (!browser || this._persistingStarted) return;
		this._persistingStarted = true;
		const schedule = () => {
			if (this._persistTimer !== null) return;
			this._persistTimer = setTimeout(() => {
				this._persistTimer = null;
				this._flushSnapshot();
			}, 750);
		};
		this.state.subscribe(schedule);
		this.displayMode.subscribe(schedule);
		this.playlist.subscribe(schedule);
	}

	private _flushSnapshot(): void {
		const st = get(this.state);
		const dm = get(this.displayMode);
		const pl = get(this.playlist);
		// Only persist while the bottom-right player is the active surface and
		// there's a track loaded — otherwise we'd overwrite a still-relevant
		// snapshot whenever some other route's `'inline'` player shows up.
		// `stop()` is the explicit clearer.
		if (!st.currentFile || dm !== 'navbar') return;
		this.snapshot.set({
			displayMode: dm,
			currentFile: st.currentFile,
			positionSecs: st.positionSecs,
			directStreamUrl: st.directStreamUrl,
			directStreamMimeType: st.directStreamMimeType,
			firkinId: st.firkinId,
			trackId: st.trackId,
			trackTitle: st.trackTitle,
			syncedLyrics: st.syncedLyrics,
			playlist: pl
		});
	}

	getSnapshot(): PlayerSnapshot {
		return get(this.snapshot);
	}

	setPendingSeek(secs: number | null): void {
		this.state.update((s) => ({ ...s, pendingSeekSecs: secs }));
	}

	// ===== Direct URL playback (yt-dlp / torrent stream / IPFS gateway) =====

	/// Surface the currently-selected track in the floating panel **before**
	/// the stream URL has been resolved, so the user gets immediate feedback
	/// while the upstream resolver (yt-dlp / etc.) does its work. The eventual
	/// `playUrl()` call upgrades this same currentFile in place — when the
	/// file ids match, `playUrl` skips its `stop()` reset so the loading state
	/// flows directly into the streaming state without a flicker.
	async beginLoad(
		file: PlayableFile,
		displayMode?: PlayerDisplayMode,
		syncedLyrics?: SubsLyricsSyncedLine[] | null
	): Promise<void> {
		if (!browser) return;

		this._resetPlaybackState();
		this.playGeneration++;
		if (displayMode) this.displayMode.set(displayMode);

		this.state.update((s) => ({
			...s,
			currentFile: file,
			connectionState: 'connecting',
			error: null,
			positionSecs: 0,
			durationSecs: file.durationSeconds,
			isPaused: true,
			buffering: true,
			directStreamUrl: null,
			directStreamMimeType: null,
			streamOffsetSecs: 0,
			firkinId: null,
			trackId: null,
			trackTitle: null,
			syncedLyrics: syncedLyrics && syncedLyrics.length > 0 ? syncedLyrics : null
		}));
	}

	async playUrl(
		file: PlayableFile,
		streamUrl: string,
		mimeType?: string | null,
		displayMode?: PlayerDisplayMode,
		firkinId?: string | null,
		syncedLyrics?: SubsLyricsSyncedLine[] | null,
		trackId?: string | null,
		trackTitle?: string | null,
		streamOffsetSecs?: number
	): Promise<void> {
		if (!browser) return;

		// When upgrading from a `beginLoad` of the same file (audio callers
		// surface the panel immediately while the stream URL resolves), skip
		// the reset so the in-flight loading state isn't wiped.
		const cur = get(this.state);
		const sameFile = cur.currentFile !== null && cur.currentFile.id === file.id;
		if (!sameFile) {
			this._resetPlaybackState();
		}
		this.playGeneration++;
		if (displayMode) this.displayMode.set(displayMode);

		const offsetSecs = streamOffsetSecs && streamOffsetSecs > 0 ? streamOffsetSecs : 0;
		this.state.update((s) => ({
			...s,
			currentFile: file,
			connectionState: 'streaming',
			error: null,
			positionSecs: offsetSecs,
			durationSecs: file.durationSeconds,
			isPaused: false,
			buffering: false,
			directStreamUrl: streamUrl,
			directStreamMimeType: mimeType ?? null,
			streamOffsetSecs: offsetSecs,
			firkinId: firkinId ?? null,
			trackId: trackId ?? null,
			trackTitle: trackTitle ?? null,
			syncedLyrics: syncedLyrics && syncedLyrics.length > 0 ? syncedLyrics : null
		}));
	}

	setBuffering(buffering: boolean): void {
		// Short-circuit at the caller — Svelte writable stores fire
		// `safe_not_equal` which treats every object reference as changed,
		// so returning the same `s` from `update()` still notifies every
		// subscriber and cascades through the reactive graph.
		const cur = get(this.state);
		if (cur.buffering === buffering) return;
		this.state.update((s) => ({ ...s, buffering }));
	}

	// ===== Seeking — direct URL playback drives the element directly via PlayerVideo =====

	seek(positionSecs: number): void {
		if (this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
		}

		this.state.update((s) => ({
			...s,
			positionSecs,
			isSeeking: true
		}));

		this.seekTimeout = setTimeout(() => {
			this.seekTimeout = null;
			this.state.update((s) => ({ ...s, isSeeking: false }));
		}, 500);
	}

	setSeeking(isSeeking: boolean): void {
		if (isSeeking && this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
			this.seekTimeout = null;
		}
		this.state.update((s) => ({ ...s, isSeeking }));
	}

	// ===== Playback controls =====

	setPaused(isPaused: boolean): void {
		const cur = get(this.state);
		if (cur.isPaused === isPaused) return;
		this.state.update((s) => ({ ...s, isPaused }));
	}

	setVolume(volume: number): void {
		this.updateSettings({ preferredVolume: volume });
	}

	getVolume(): number {
		return this.get().preferredVolume;
	}

	// ===== Stop playback =====

	private _resetPlaybackState(): void {
		if (this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
			this.seekTimeout = null;
		}

		this.state.update((s) => ({
			...s,
			currentFile: null,
			connectionState: 'idle',
			error: null,
			positionSecs: 0,
			durationSecs: null,
			isSeeking: false,
			isPaused: true,
			buffering: false,
			directStreamUrl: null,
			directStreamMimeType: null,
			streamOffsetSecs: 0,
			firkinId: null,
			trackId: null,
			trackTitle: null,
			syncedLyrics: null,
			pendingSeekSecs: null
		}));
		this.displayMode.set('fullscreen');
	}

	async stop(): Promise<void> {
		this._resetPlaybackState();
		this.playlist.set(null);
		if (browser) {
			if (this._persistTimer !== null) {
				clearTimeout(this._persistTimer);
				this._persistTimer = null;
			}
			this.snapshot.set({ ...initialSnapshot });
		}
	}

	// ===== Settings =====

	updateSettings(updates: Partial<PlayerSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== Lifecycle =====
	//
	// `destroy()` runs from the root layout's `onDestroy` — which fires on
	// page refresh as part of the SPA teardown. Tearing down to `stop()`
	// would wipe the snapshot before the next boot has a chance to restore
	// it, so we only flush in-flight timers here. The snapshot is cleared
	// only by an explicit user-driven `stop()` (the X button).

	async destroy(): Promise<void> {
		if (this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
			this.seekTimeout = null;
		}
		if (this._persistTimer !== null) {
			clearTimeout(this._persistTimer);
			// Final synchronous flush so the latest position lands in localStorage
			// even if the unload races the throttle window.
			this._persistTimer = null;
			this._flushSnapshot();
		}
	}
}

export const playerService = new PlayerService();
