import { browser } from '$app/environment';

export const isTauri: boolean =
	browser &&
	typeof window !== 'undefined' &&
	('__TAURI__' in window ||
		window.location.origin.includes('tauri.localhost') ||
		window.location.origin.startsWith('tauri://'));

export const isMobile: boolean =
	isTauri && browser && typeof navigator !== 'undefined' && /android/i.test(navigator.userAgent);
