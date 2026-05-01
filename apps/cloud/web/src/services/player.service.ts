import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import type {
	PlayerSettings,
	PlayerState,
	PlayerDisplayMode,
	PlayableFile
} from '$types/player.type';

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
	awaitingPlay: false
};

class PlayerService extends ObjectServiceClass<PlayerSettings> {
	public state: Writable<PlayerState> = writable(initialState);
	public displayMode: Writable<PlayerDisplayMode> = writable('fullscreen');

	setDisplayMode(mode: PlayerDisplayMode): void {
		this.displayMode.set(mode);
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

	async playUrl(
		file: PlayableFile,
		streamUrl: string,
		mimeType?: string | null,
		displayMode?: PlayerDisplayMode,
		firkinId?: string | null,
		options?: { autoplay?: boolean }
	): Promise<void> {
		if (!browser) return;

		await this.stop();
		this.playGeneration++;
		if (displayMode) this.displayMode.set(displayMode);

		const autoplay = options?.autoplay !== false;

		this.state.update((s) => ({
			...s,
			currentFile: file,
			connectionState: 'streaming',
			error: null,
			positionSecs: 0,
			durationSecs: file.durationSeconds,
			isPaused: !autoplay,
			buffering: false,
			directStreamUrl: streamUrl,
			directStreamMimeType: mimeType ?? null,
			firkinId: firkinId ?? null,
			awaitingPlay: !autoplay
		}));
	}

	setBuffering(buffering: boolean): void {
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
		this.state.update((s) => ({ ...s, isPaused }));
	}

	setVolume(volume: number): void {
		this.updateSettings({ preferredVolume: volume });
	}

	getVolume(): number {
		return this.get().preferredVolume;
	}

	// ===== Stop playback =====

	async stop(): Promise<void> {
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
			awaitingPlay: false
		}));
		this.displayMode.set('fullscreen');
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
