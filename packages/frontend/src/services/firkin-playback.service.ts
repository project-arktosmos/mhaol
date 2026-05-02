import { writable, type Writable } from 'svelte/store';
import type { CloudFirkin, FirkinFile } from '$types/firkin.type';

export interface FirkinPlaybackState {
	firkin: CloudFirkin | null;
	files: FirkinFile[];
	/// Currently-playing file's value (CID for IPFS files), or null when
	/// nothing is playing or the user has stopped playback.
	currentFile: string | null;
}

const initialState: FirkinPlaybackState = {
	firkin: null,
	files: [],
	currentFile: null
};

class FirkinPlaybackService {
	state: Writable<FirkinPlaybackState> = writable(initialState);

	select(firkin: CloudFirkin): void {
		const files = (firkin.files ?? []).filter((f) => f.type === 'ipfs');
		this.state.set({ firkin, files, currentFile: null });
	}

	clear(): void {
		this.state.set(initialState);
	}
}

export const firkinPlaybackService = new FirkinPlaybackService();
