import { browser } from '$app/environment';

function getApiBase(): string {
	if (!browser) return '';
	return localStorage.getItem('api-server-url') || `http://${window.location.hostname}:1530`;
}

export const apiBase = getApiBase();

export function apiUrl(path: string): string {
	return `${apiBase}${path}`;
}
