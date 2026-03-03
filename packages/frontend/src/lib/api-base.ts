import { browser } from '$app/environment';

function getApiBase(): string {
	if (!browser) return '';
	return localStorage.getItem('api-server-url') || '';
}

export const apiBase = getApiBase();

export function apiUrl(path: string): string {
	return `${apiBase}${path}`;
}
