import { browser } from '$app/environment';
import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { playerService } from 'ui-lib/services/player.service';
import type { DocumentFile } from 'ui-lib/types/document.type';

interface StartSessionResponse {
	sessionId: string;
	roomId: string;
	signalingUrl: string;
}

const VIDEO_EXTENSIONS = /\.(mp4|m4v|mkv|webm|mov|avi|ts|mpe?g|wmv|flv|ogv)$/i;
const AUDIO_EXTENSIONS = /\.(mp3|flac|m4a|aac|alac|ogg|oga|opus|wav|wma|aiff|aif|ape)$/i;

export function isVideoFile(file: DocumentFile): boolean {
	const label = file.title ?? file.value ?? '';
	return VIDEO_EXTENSIONS.test(label);
}

export function isAudioFile(file: DocumentFile): boolean {
	const label = file.title ?? file.value ?? '';
	return AUDIO_EXTENSIONS.test(label);
}

export function isPlayableFile(file: DocumentFile): boolean {
	return isVideoFile(file) || isAudioFile(file);
}

class DocumentStreamService {
	async play(file: DocumentFile): Promise<void> {
		if (!browser) return;
		if (file.type !== 'ipfs') return;

		const mode: 'audio' | 'video' = isAudioFile(file) ? 'audio' : 'video';

		const session = await fetchJson<StartSessionResponse>('/api/p2p-stream/sessions', {
			method: 'POST',
			body: JSON.stringify({ cid: file.value, mode })
		});

		const name = file.title ?? file.value;
		await playerService.playRemote(
			name,
			session.sessionId,
			session.roomId,
			session.signalingUrl,
			mode
		);
	}
}

export const documentStreamService = new DocumentStreamService();
