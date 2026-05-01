import { CID } from 'multiformats/cid';
import { createFile, type Mp4BoxFile } from 'mp4box';
import type { PlayerIpfsClient } from './client';

/**
 * MSE-fed streaming pipeline for firkin `ipfs` files. Pulls UnixFS chunks
 * from Helia as they arrive and feeds them into a `<video>` element via a
 * `MediaSource`, so playback starts as soon as the first segment is
 * decodable instead of waiting for the entire file to download.
 *
 * Three modes, picked by container:
 *   - `mp4`/`m4v`: drive an `mp4box.js` re-muxer so the `<video>` sees
 *     fragmented MP4 segments. Required because most `.mp4` files
 *     produced by torrent rippers / yt-dlp / etc. are *unfragmented*
 *     and `MediaSource.appendBuffer` rejects them.
 *   - `webm`: direct-feed bytes into a SourceBuffer typed
 *     `video/webm; codecs="vp9,opus"` (or `vp8,vorbis` as a fallback).
 *   - anything else (mkv, mov, avi, raw audio): fall back to the old
 *     full-buffer Blob URL — most of these aren't natively playable in
 *     any browser, so streaming wouldn't change the outcome.
 *
 * The pipeline is fully cancellable via the supplied `AbortSignal` —
 * cancellation aborts the UnixFS read, tears down the MediaSource, and
 * revokes the object URL.
 */

export type StreamPlayerKind = 'mse-mp4' | 'mse-webm' | 'blob';

export interface StreamPlayerProgress {
	bytesReceived: number;
	mode: StreamPlayerKind;
	/** Best-known total — only known once the UnixFS root reports a size. */
	totalBytes?: number;
}

export interface StreamPlayerHandle {
	/** Object URL assigned to the `<video>`/`<audio>` `src`. */
	src: string;
	mode: StreamPlayerKind;
	/** Resolves when the underlying source has finished feeding the video element. */
	done: Promise<void>;
	/** Cancel any in-flight network/decoder work and revoke the URL. */
	cancel(): void;
}

export interface StartStreamOptions {
	client: PlayerIpfsClient;
	/** UnixFS CID of the file to play. */
	cid: string;
	/** File title — used to pick the container/codec mode by extension. */
	title: string | undefined;
	/** Receives running totals as bytes arrive. */
	onProgress?: (p: StreamPlayerProgress) => void;
	/** Optional caller-supplied abort signal. */
	signal?: AbortSignal;
}

export function pickStreamMode(title: string | undefined): StreamPlayerKind {
	const t = (title ?? '').toLowerCase();
	if (t.endsWith('.mp4') || t.endsWith('.m4v')) return 'mse-mp4';
	if (t.endsWith('.webm')) return 'mse-webm';
	return 'blob';
}

function guessMime(title: string | undefined): string {
	const t = (title ?? '').toLowerCase();
	if (t.endsWith('.mp4') || t.endsWith('.m4v')) return 'video/mp4';
	if (t.endsWith('.webm')) return 'video/webm';
	if (t.endsWith('.mov')) return 'video/quicktime';
	if (t.endsWith('.mkv')) return 'video/x-matroska';
	if (t.endsWith('.mp3')) return 'audio/mpeg';
	if (t.endsWith('.flac')) return 'audio/flac';
	if (t.endsWith('.ogg')) return 'audio/ogg';
	if (t.endsWith('.m4a')) return 'audio/mp4';
	if (t.endsWith('.opus')) return 'audio/opus';
	return 'video/mp4';
}

export async function startStream(opts: StartStreamOptions): Promise<StreamPlayerHandle> {
	const ctrl = new AbortController();
	const signal = opts.signal ? linkSignals(opts.signal, ctrl.signal) : ctrl.signal;

	const mode = pickStreamMode(opts.title);

	if (mode === 'mse-mp4' && typeof MediaSource !== 'undefined') {
		try {
			return await startMp4Stream(opts, ctrl, signal);
		} catch (err) {
			console.warn('[stream-player] mp4 streaming failed, falling back to blob', err);
		}
	}

	if (mode === 'mse-webm' && typeof MediaSource !== 'undefined') {
		try {
			return await startWebmStream(opts, ctrl, signal);
		} catch (err) {
			console.warn('[stream-player] webm streaming failed, falling back to blob', err);
		}
	}

	return startBlobStream(opts, ctrl, signal);
}

/* ----------------------------- mp4 (fragmented) ---------------------------- */

async function startMp4Stream(
	opts: StartStreamOptions,
	ctrl: AbortController,
	signal: AbortSignal
): Promise<StreamPlayerHandle> {
	const ms = new MediaSource();
	const url = URL.createObjectURL(ms);

	await waitForEvent(ms, 'sourceopen', signal);

	const mp4: Mp4BoxFile = createFile();
	let sourceBuffer: SourceBuffer | null = null;
	const queue: ArrayBuffer[] = [];
	let appending = false;
	let finished = false;
	let lastError: unknown = null;

	async function drain() {
		if (appending || !sourceBuffer) return;
		appending = true;
		try {
			while (queue.length > 0 && !signal.aborted && !finished) {
				const seg = queue.shift()!;
				await appendOnce(sourceBuffer, seg, signal);
			}
		} catch (err) {
			lastError = err;
		} finally {
			appending = false;
		}
	}

	const ready = new Promise<void>((resolve, reject) => {
		mp4.onReady = (info) => {
			try {
				const codecs = info.tracks
					.map((t) => t.codec)
					.filter((c): c is string => Boolean(c))
					.join(',');
				const mime = `video/mp4; codecs="${codecs}"`;
				if (!MediaSource.isTypeSupported(mime)) {
					reject(new Error(`MediaSource refused ${mime}`));
					return;
				}
				sourceBuffer = ms.addSourceBuffer(mime);
				sourceBuffer.mode = 'segments';
				for (const t of info.tracks) {
					mp4.setSegmentOptions(t.id, null, { nbSamples: 60 });
				}
				const initSegments = mp4.initializeSegmentation();
				for (const seg of initSegments) {
					queue.push(seg.buffer);
				}
				void drain();
				mp4.start();
				resolve();
			} catch (err) {
				reject(err);
			}
		};
		mp4.onError = (msg) => reject(new Error(msg || 'mp4box error'));
		mp4.onSegment = (_id, _user, buffer, _sampleNum, _last) => {
			queue.push(buffer);
			void drain();
		};
	});

	const done = (async () => {
		let bytes = 0;
		let offset = 0;
		try {
			const cid = CID.parse(opts.cid);
			for await (const chunk of opts.client.fs.cat(cid, { signal })) {
				if (signal.aborted) throw new DOMException('aborted', 'AbortError');
				const ab = new ArrayBuffer(chunk.byteLength);
				new Uint8Array(ab).set(chunk);
				const tagged = ab as ArrayBuffer & { fileStart: number };
				tagged.fileStart = offset;
				offset += chunk.byteLength;
				mp4.appendBuffer(tagged);
				bytes += chunk.byteLength;
				opts.onProgress?.({ bytesReceived: bytes, mode: 'mse-mp4' });
			}
			mp4.flush();
			finished = true;
			while (queue.length > 0 || appending) {
				if (signal.aborted) break;
				await new Promise((r) => setTimeout(r, 30));
			}
			if (lastError) throw lastError;
			if (ms.readyState === 'open' && !signal.aborted) {
				try {
					ms.endOfStream();
				} catch {
					// ignore — buffers may have desynced; player still has what it has
				}
			}
		} finally {
			finished = true;
		}
	})();

	// Surface ready-vs-stream-failure: if mp4box never reaches onReady, the
	// outer caller should fall back to blob mode.
	const guarded = Promise.race([
		ready,
		done.then(() => {
			if (!sourceBuffer) {
				throw new Error('mp4box never produced an init segment');
			}
		})
	]);
	await guarded.catch((err) => {
		ctrl.abort();
		URL.revokeObjectURL(url);
		throw err;
	});

	return {
		src: url,
		mode: 'mse-mp4',
		done,
		cancel() {
			ctrl.abort();
			try {
				if (ms.readyState === 'open') ms.endOfStream();
			} catch {
				// ignore
			}
			URL.revokeObjectURL(url);
		}
	};
}

/* --------------------------------- webm ----------------------------------- */

async function startWebmStream(
	opts: StartStreamOptions,
	ctrl: AbortController,
	signal: AbortSignal
): Promise<StreamPlayerHandle> {
	const ms = new MediaSource();
	const url = URL.createObjectURL(ms);

	await waitForEvent(ms, 'sourceopen', signal);

	let mime = 'video/webm; codecs="vp9,opus"';
	if (!MediaSource.isTypeSupported(mime)) {
		mime = 'video/webm; codecs="vp8,vorbis"';
	}
	if (!MediaSource.isTypeSupported(mime)) {
		URL.revokeObjectURL(url);
		throw new Error('No supported webm codec');
	}
	const sourceBuffer = ms.addSourceBuffer(mime);

	const done = (async () => {
		let bytes = 0;
		try {
			const cid = CID.parse(opts.cid);
			for await (const chunk of opts.client.fs.cat(cid, { signal })) {
				if (signal.aborted) throw new DOMException('aborted', 'AbortError');
				const ab = new ArrayBuffer(chunk.byteLength);
				new Uint8Array(ab).set(chunk);
				await appendOnce(sourceBuffer, ab, signal);
				bytes += chunk.byteLength;
				opts.onProgress?.({ bytesReceived: bytes, mode: 'mse-webm' });
			}
			if (ms.readyState === 'open') {
				try {
					ms.endOfStream();
				} catch {
					// ignore
				}
			}
		} catch (err) {
			if (!signal.aborted) throw err;
		}
	})();

	return {
		src: url,
		mode: 'mse-webm',
		done,
		cancel() {
			ctrl.abort();
			try {
				if (ms.readyState === 'open') ms.endOfStream();
			} catch {
				// ignore
			}
			URL.revokeObjectURL(url);
		}
	};
}

/* --------------------------------- blob ----------------------------------- */

async function startBlobStream(
	opts: StartStreamOptions,
	ctrl: AbortController,
	signal: AbortSignal
): Promise<StreamPlayerHandle> {
	const cid = CID.parse(opts.cid);
	const chunks: Uint8Array[] = [];
	let bytes = 0;
	for await (const chunk of opts.client.fs.cat(cid, { signal })) {
		if (signal.aborted) throw new DOMException('aborted', 'AbortError');
		chunks.push(chunk);
		bytes += chunk.byteLength;
		opts.onProgress?.({ bytesReceived: bytes, mode: 'blob' });
	}
	const merged = new Uint8Array(bytes);
	let offset = 0;
	for (const c of chunks) {
		merged.set(c, offset);
		offset += c.byteLength;
	}
	const blob = new Blob([merged], { type: guessMime(opts.title) });
	const url = URL.createObjectURL(blob);
	return {
		src: url,
		mode: 'blob',
		done: Promise.resolve(),
		cancel() {
			ctrl.abort();
			URL.revokeObjectURL(url);
		}
	};
}

/* -------------------------------- helpers --------------------------------- */

function appendOnce(sb: SourceBuffer, buffer: ArrayBuffer, signal: AbortSignal): Promise<void> {
	return new Promise((resolve, reject) => {
		if (signal.aborted) {
			reject(new DOMException('aborted', 'AbortError'));
			return;
		}
		const onUpdateEnd = () => {
			cleanup();
			resolve();
		};
		const onError = () => {
			cleanup();
			reject(new Error('SourceBuffer append failed'));
		};
		const onAbort = () => {
			cleanup();
			reject(new DOMException('aborted', 'AbortError'));
		};
		const cleanup = () => {
			sb.removeEventListener('updateend', onUpdateEnd);
			sb.removeEventListener('error', onError);
			signal.removeEventListener('abort', onAbort);
		};
		sb.addEventListener('updateend', onUpdateEnd, { once: true });
		sb.addEventListener('error', onError, { once: true });
		signal.addEventListener('abort', onAbort, { once: true });
		try {
			sb.appendBuffer(buffer);
		} catch (err) {
			cleanup();
			reject(err);
		}
	});
}

function waitForEvent(target: EventTarget, name: string, signal: AbortSignal): Promise<void> {
	return new Promise((resolve, reject) => {
		const onEvent = () => {
			cleanup();
			resolve();
		};
		const onAbort = () => {
			cleanup();
			reject(new DOMException('aborted', 'AbortError'));
		};
		const cleanup = () => {
			target.removeEventListener(name, onEvent);
			signal.removeEventListener('abort', onAbort);
		};
		target.addEventListener(name, onEvent, { once: true });
		signal.addEventListener('abort', onAbort, { once: true });
	});
}

function linkSignals(parent: AbortSignal, child: AbortSignal): AbortSignal {
	if (parent.aborted) return parent;
	const linked = new AbortController();
	const onParent = () => linked.abort(parent.reason);
	const onChild = () => linked.abort(child.reason);
	parent.addEventListener('abort', onParent, { once: true });
	child.addEventListener('abort', onChild, { once: true });
	return linked.signal;
}
