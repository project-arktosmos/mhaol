import { browser } from '$app/environment';
import { isTauri } from './platform';

const STORAGE_KEY = 'mhaol-api-base';
const SUBSCRIBERS = new Set<(value: string) => void>();

function readOverride(): string | null {
	if (!browser) return null;
	try {
		return localStorage.getItem(STORAGE_KEY);
	} catch {
		return null;
	}
}

function defaultBase(): string {
	if (!browser) return '';
	// Tauri shells (cloud tray, android-tv, android-mobile) load the SPA from a
	// non-HTTP origin (`tauri://localhost`, `http://tauri.localhost`), so a
	// relative `/api/...` would never reach the backend. Default to the local
	// loopback bind that the embedded backend uses (android-mobile, cloud bin).
	// Android TV has no backend on-device — the user must point this at a
	// reachable cloud via the Settings page.
	if (isTauri) return 'http://127.0.0.1:9898';
	// Plain browser → same-origin (the backend serves the SPA itself).
	return '';
}

export function getApiBase(): string {
	const override = readOverride();
	if (override !== null) return override;
	return defaultBase();
}

export function getDefaultApiBase(): string {
	return defaultBase();
}

export function setApiBase(url: string): void {
	if (!browser) return;
	const trimmed = url.trim().replace(/\/+$/, '');
	try {
		if (trimmed.length === 0) {
			localStorage.removeItem(STORAGE_KEY);
		} else {
			localStorage.setItem(STORAGE_KEY, trimmed);
		}
	} catch {
		/* storage may be unavailable; ignore */
	}
	const next = getApiBase();
	for (const sub of SUBSCRIBERS) sub(next);
}

export function subscribeApiBase(callback: (value: string) => void): () => void {
	SUBSCRIBERS.add(callback);
	return () => SUBSCRIBERS.delete(callback);
}

export function apiUrl(path: string): string {
	const base = getApiBase();
	if (!base) return path;
	if (/^https?:\/\//i.test(path)) return path;
	return `${base}${path.startsWith('/') ? path : `/${path}`}`;
}
