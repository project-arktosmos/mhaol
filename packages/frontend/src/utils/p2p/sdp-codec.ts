import { deflateSync, inflateSync, strToU8, strFromU8 } from 'fflate';
import type { P2pSignalingPayload } from '$types/p2p.type';

export function encodePayload(payload: P2pSignalingPayload): string {
	const json = JSON.stringify(payload);
	const compressed = deflateSync(strToU8(json), { level: 9 });
	return uint8ToBase64(compressed);
}

export function decodePayload(encoded: string): P2pSignalingPayload {
	const compressed = base64ToUint8(encoded.trim());
	const json = strFromU8(inflateSync(compressed));
	return JSON.parse(json) as P2pSignalingPayload;
}

export function fitsInQrCode(encoded: string): boolean {
	return encoded.length <= 4296;
}

function uint8ToBase64(bytes: Uint8Array): string {
	let binary = '';
	for (let i = 0; i < bytes.length; i++) {
		binary += String.fromCharCode(bytes[i]);
	}
	return btoa(binary);
}

function base64ToUint8(base64: string): Uint8Array {
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) {
		bytes[i] = binary.charCodeAt(i);
	}
	return bytes;
}
