import { browser } from '$app/environment';
import { get } from 'svelte/store';
import { playerService } from '$services/player.service';
import { userIdentityService } from '$lib/user-identity.service';

const HEARTBEAT_INTERVAL_MS = 10_000;
const HEARTBEAT_DELTA_SECONDS = HEARTBEAT_INTERVAL_MS / 1000;

export interface MediaTracker {
	id: string;
	firkinId: string;
	trackId?: string;
	trackTitle?: string;
	address: string;
	totalSeconds: number;
	last_played_at: string;
	created_at: string;
	updated_at: string;
}

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

export async function listMediaTrackers(address?: string): Promise<MediaTracker[]> {
	const params = new URLSearchParams();
	if (address) params.set('address', address);
	const qs = params.toString();
	const url = qs ? `/api/media-trackers?${qs}` : '/api/media-trackers';
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as MediaTracker[];
}

class MediaTrackerService {
	private interval: ReturnType<typeof setInterval> | null = null;
	private currentFirkinId: string | null = null;
	private currentTrackId: string | null = null;
	private currentTrackTitle: string | null = null;
	private initialized = false;

	initialize(): void {
		if (!browser || this.initialized) return;
		this.initialized = true;

		// Drive the tracker off the player service: any playUrl caller that
		// passes a firkin id (catalog detail page's IPFS HLS + torrent stream
		// flows, plus the navbar music playlist) sets `firkinId` on the
		// player state. When the playlist also surfaces a `trackId`, the
		// heartbeat buckets time per-track within the album. Flipping the
		// firkin or track restarts the heartbeat loop so swaps don't bleed
		// time into the wrong row. The interval-side `sendHeartbeat` already
		// short-circuits when paused / not streaming.
		playerService.state.subscribe((s) => {
			const firkinId = s.firkinId;
			const trackId = s.trackId;
			const trackTitle = s.trackTitle;
			if (
				firkinId === this.currentFirkinId &&
				trackId === this.currentTrackId &&
				trackTitle === this.currentTrackTitle
			) {
				return;
			}
			this.stop();
			if (firkinId) {
				this.start(firkinId, trackId, trackTitle);
			}
		});

		window.addEventListener('pagehide', this.handlePageHide);
	}

	private handlePageHide = (): void => {
		this.stop();
	};

	private start(firkinId: string, trackId: string | null, trackTitle: string | null): void {
		this.currentFirkinId = firkinId;
		this.currentTrackId = trackId;
		this.currentTrackTitle = trackTitle;
		void this.sendHeartbeat(firkinId, trackId, trackTitle, 0);
		this.interval = setInterval(() => {
			void this.sendHeartbeat(firkinId, trackId, trackTitle, HEARTBEAT_DELTA_SECONDS);
		}, HEARTBEAT_INTERVAL_MS);
	}

	private stop(): void {
		if (this.interval !== null) {
			clearInterval(this.interval);
			this.interval = null;
		}
		this.currentFirkinId = null;
		this.currentTrackId = null;
		this.currentTrackTitle = null;
	}

	private async sendHeartbeat(
		firkinId: string,
		trackId: string | null,
		trackTitle: string | null,
		deltaSeconds: number
	): Promise<void> {
		const identity = get(userIdentityService.state).identity;
		if (!identity) return;

		// Recurring heartbeats only count when the player is actively
		// streaming and not paused — pauses, buffering, and connection
		// teardown all stop time accruing. The play-start beat (delta 0) is
		// always sent so the tracker row exists from the first frame.
		if (deltaSeconds > 0) {
			const playerState = get(playerService.state);
			if (playerState.isPaused || playerState.connectionState !== 'streaming') {
				return;
			}
		}

		try {
			await fetch('/api/media-trackers/heartbeat', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({
					firkinId,
					trackId: trackId ?? undefined,
					trackTitle: trackTitle ?? undefined,
					address: identity.address,
					deltaSeconds
				})
			});
		} catch {
			// Best-effort: the next tick re-tries with another delta.
		}
	}
}

export const mediaTrackerService = new MediaTrackerService();
