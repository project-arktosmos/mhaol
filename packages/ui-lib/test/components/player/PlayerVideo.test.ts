// Component-level tests for PlayerVideo's direct-URL playback path. The
// failure mode this test pins is the one that kept biting in the cloud
// /youtube page: position/duration never made it back into
// `playerService.state` so the seek bar stayed stuck at 0:00, and seeks the
// user dragged on the bar did nothing because `playerService.seek()`
// short-circuits when there is no WebRTC data channel.
//
// Both audio and video YouTube playback go through the same
// `directStreamUrl` code path (PlayerVideo always renders a single `<video>`
// element since the audio-element split was removed), so a single set of
// assertions covers both modes.

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { get } from 'svelte/store';
import PlayerVideo from '../../../src/components/player/PlayerVideo.svelte';
import { playerService } from '../../../src/services/player.service';
import type { PlayableFile } from '../../../src/types/player.type';

const STREAM_URL = 'https://googlevideo.example/audio.mp4';

function makeFile(overrides: Partial<PlayableFile> = {}): PlayableFile {
	return {
		id: 'youtube:test:audio',
		type: 'youtube',
		name: 'Test Track',
		outputPath: '',
		mode: 'audio',
		format: null,
		videoFormat: null,
		thumbnailUrl: null,
		durationSeconds: null,
		size: 0,
		completedAt: '',
		...overrides
	};
}

// happy-dom doesn't fully implement HTMLMediaElement: `play()` throws and
// `currentTime`/`duration` aren't writable. Patch enough of the prototype
// to look like a real element to the component.
function stubMediaElement() {
	const proto = HTMLMediaElement.prototype as unknown as Record<string, unknown>;
	proto.play = vi.fn(() => Promise.resolve());
	proto.pause = vi.fn();
	proto.load = vi.fn();
	let _currentTime = 0;
	let _duration = NaN;
	Object.defineProperty(HTMLMediaElement.prototype, 'currentTime', {
		configurable: true,
		get() {
			return _currentTime;
		},
		set(v: number) {
			_currentTime = v;
		}
	});
	Object.defineProperty(HTMLMediaElement.prototype, 'duration', {
		configurable: true,
		get() {
			return _duration;
		},
		set(v: number) {
			_duration = v;
		}
	});
	Object.defineProperty(HTMLMediaElement.prototype, 'paused', {
		configurable: true,
		get() {
			return true;
		}
	});
	Object.defineProperty(HTMLMediaElement.prototype, 'volume', {
		configurable: true,
		get() {
			return 1;
		},
		set() {
			// no-op
		}
	});
	Object.defineProperty(HTMLMediaElement.prototype, 'muted', {
		configurable: true,
		get() {
			return false;
		},
		set() {
			// no-op
		}
	});
}

function resetPlayerState() {
	playerService.state.set({
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
		directStreamMimeType: null
	});
}

async function flush() {
	// One macrotask + one microtask is enough to let Svelte's effect graph
	// settle after a prop change.
	await new Promise((r) => setTimeout(r, 0));
	await Promise.resolve();
}

describe.each([
	{ mode: 'audio' as const, label: 'audio' },
	{ mode: 'video' as const, label: 'video' }
])('PlayerVideo direct-URL playback ($label mode)', ({ mode }) => {
	beforeEach(() => {
		stubMediaElement();
		resetPlayerState();
	});

	afterEach(() => {
		resetPlayerState();
	});

	it('pumps element.duration into playerService.state.durationSecs on loadedmetadata', async () => {
		const { container } = render(PlayerVideo, {
			props: {
				file: makeFile({ mode }),
				connectionState: 'streaming',
				positionSecs: 0,
				durationSecs: null,
				buffering: false,
				directStreamUrl: STREAM_URL
			}
		});
		await flush();

		const video = container.querySelector('video') as HTMLVideoElement | null;
		expect(video, 'PlayerVideo must render a <video> element').not.toBeNull();
		if (!video) return;

		(video as unknown as { duration: number }).duration = 213.4;
		await fireEvent(video, new Event('loadedmetadata'));
		await flush();

		expect(get(playerService.state).durationSecs).toBeCloseTo(213.4, 5);
	});

	it('pumps element.currentTime into playerService.state.positionSecs on timeupdate', async () => {
		const { container } = render(PlayerVideo, {
			props: {
				file: makeFile({ mode }),
				connectionState: 'streaming',
				positionSecs: 0,
				durationSecs: null,
				buffering: false,
				directStreamUrl: STREAM_URL
			}
		});
		await flush();

		const video = container.querySelector('video') as HTMLVideoElement | null;
		expect(video).not.toBeNull();
		if (!video) return;

		(video as unknown as { currentTime: number }).currentTime = 42.7;
		await fireEvent(video, new Event('timeupdate'));
		await flush();

		expect(get(playerService.state).positionSecs).toBeCloseTo(42.7, 5);

		// Another tick should keep updating, not get stuck.
		(video as unknown as { currentTime: number }).currentTime = 99;
		await fireEvent(video, new Event('timeupdate'));
		await flush();
		expect(get(playerService.state).positionSecs).toBe(99);
	});

	it('does NOT overwrite positionSecs from timeupdate while the user is seeking', async () => {
		const { container } = render(PlayerVideo, {
			props: {
				file: makeFile({ mode }),
				connectionState: 'streaming',
				positionSecs: 0,
				durationSecs: null,
				buffering: false,
				directStreamUrl: STREAM_URL
			}
		});
		await flush();

		const video = container.querySelector('video') as HTMLVideoElement | null;
		expect(video).not.toBeNull();
		if (!video) return;

		// Simulate the user grabbing the seek bar — sets isSeeking on the store.
		playerService.setSeeking(true);
		(video as unknown as { currentTime: number }).currentTime = 5;
		await fireEvent(video, new Event('timeupdate'));
		await flush();

		// Position must NOT track the element while the user is dragging.
		expect(get(playerService.state).positionSecs).toBe(0);
	});

	it('seeking via the seek bar writes element.currentTime AND publishes the new position', async () => {
		// Pass `durationSecs: 200` directly — the seek bar guards on
		// `durationSecs > 0` and we want the seek interaction itself to be
		// the thing under test, not the parent's prop wiring.
		const { container } = render(PlayerVideo, {
			props: {
				file: makeFile({ mode }),
				connectionState: 'streaming',
				positionSecs: 0,
				durationSecs: 200,
				buffering: false,
				directStreamUrl: STREAM_URL
			}
		});
		await flush();

		const video = container.querySelector('video') as HTMLVideoElement | null;
		expect(video).not.toBeNull();
		if (!video) return;

		// Find the seek bar (role="slider") and stub its bounding rect so
		// `getPositionFromEvent` can map clientX → fraction.
		const slider = container.querySelector('[role="slider"]') as HTMLElement | null;
		expect(slider, 'seek bar must be present when streaming').not.toBeNull();
		if (!slider) return;

		const rect = { left: 0, top: 0, right: 400, bottom: 10, width: 400, height: 10 };
		slider.getBoundingClientRect = () => rect as unknown as DOMRect;

		// Drag to the 25% mark = 50s. PlayerSeekBar binds `mousemove` /
		// `mouseup` on `window`, so dispatch the release event there
		// directly — `fireEvent(window, ...)` doesn't always reach the
		// global Window in happy-dom.
		await fireEvent.mouseDown(slider, { clientX: 100 });
		window.dispatchEvent(new MouseEvent('mouseup', { clientX: 100, bubbles: true }));
		await flush();

		expect(video.currentTime).toBeCloseTo(50, 5);
		expect(get(playerService.state).positionSecs).toBeCloseTo(50, 5);
		// Seek end must clear the seeking flag so timeupdate can resume.
		expect(get(playerService.state).isSeeking).toBe(false);
	});
});
