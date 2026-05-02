import { get, writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import type {
	PlayerSettings,
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
	streamServerAvailable: false,
	sessionId: null,
	localPeerId: null,
	remotePeerId: null,
	positionSecs: 0,
	durationSecs: null,
	isSeeking: false,
	isPaused: true,
	buffering: false,
	directStreamUrl: null,
	directStreamMimeType: null,
	firkinId: null,
	trackId: null,
	trackTitle: null,
	syncedLyrics: null
};

class PlayerService extends ObjectServiceClass<PlayerSettings> {
	public state: Writable<PlayerState> = writable(initialState);
	public displayMode: Writable<PlayerDisplayMode> = writable('fullscreen');
	public playlist: Writable<PlayerPlaylist | null> = writable(null);

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
		trackTitle?: string | null
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

		this.state.update((s) => ({
			...s,
			currentFile: file,
			connectionState: 'streaming',
			error: null,
			positionSecs: 0,
			durationSecs: file.durationSeconds,
			isPaused: false,
			buffering: false,
			directStreamUrl: streamUrl,
			directStreamMimeType: mimeType ?? null,
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
			sessionId: null,
			localPeerId: null,
			remotePeerId: null,
			error: null,
			positionSecs: 0,
			durationSecs: null,
			isSeeking: false,
			isPaused: true,
			buffering: false,
			directStreamUrl: null,
			directStreamMimeType: null,
			firkinId: null,
			trackId: null,
			trackTitle: null,
			syncedLyrics: null
		}));
		this.displayMode.set('fullscreen');
	}

	async stop(): Promise<void> {
		this._resetPlaybackState();
		this.playlist.set(null);
	}

	// ===== Settings =====

	updateSettings(updates: Partial<PlayerSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== Lifecycle =====

	async destroy(): Promise<void> {
		await this.stop();
	}
}

export const playerService = new PlayerService();
