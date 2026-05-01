import { browser } from '$app/environment';
import { get } from 'svelte/store';
import { firkinPlaybackService } from '$services/firkin-playback.service';
import { playerService } from '$services/player.service';
import { userIdentityService } from '$lib/user-identity.service';

const HEARTBEAT_INTERVAL_MS = 10_000;
const HEARTBEAT_DELTA_SECONDS = HEARTBEAT_INTERVAL_MS / 1000;

class MediaTrackerService {
	private interval: ReturnType<typeof setInterval> | null = null;
	private currentFirkinId: string | null = null;
	private initialized = false;

	initialize(): void {
		if (!browser || this.initialized) return;
		this.initialized = true;

		// One subscription drives the whole lifecycle: when the right-side
		// player has both a firkin selected and a file picked, we're tracking;
		// when either drops away we're not. Re-firing inside the same firkin
		// (e.g. picking a different track on an album) keeps the same row
		// since the tracker key is per-firkin, not per-file.
		firkinPlaybackService.state.subscribe((s) => {
			const firkinId = s.firkin && s.currentFile ? s.firkin.id : null;
			if (firkinId === this.currentFirkinId) return;
			this.stop();
			if (firkinId) {
				this.start(firkinId);
			}
		});

		window.addEventListener('pagehide', this.handlePageHide);
	}

	private handlePageHide = (): void => {
		this.stop();
	};

	private start(firkinId: string): void {
		this.currentFirkinId = firkinId;
		void this.sendHeartbeat(firkinId, 0);
		this.interval = setInterval(() => {
			void this.sendHeartbeat(firkinId, HEARTBEAT_DELTA_SECONDS);
		}, HEARTBEAT_INTERVAL_MS);
	}

	private stop(): void {
		if (this.interval !== null) {
			clearInterval(this.interval);
			this.interval = null;
		}
		this.currentFirkinId = null;
	}

	private async sendHeartbeat(firkinId: string, deltaSeconds: number): Promise<void> {
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
