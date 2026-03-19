import { browser } from '$app/environment';
import { isTauri } from './platform';

function getApiBase(): string {
	if (!browser) return '';

	const override = localStorage.getItem('api-server-url');
	if (override) return override;

	if (isTauri) return 'http://127.0.0.1:1530';

	return '';
}

export const apiBase = getApiBase();

export function apiUrl(path: string): string {
	return `${apiBase}${path}`;
}
