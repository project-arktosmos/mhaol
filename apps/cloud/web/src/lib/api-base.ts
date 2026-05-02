import { browser } from '$app/environment';
import { isTauri, isMobile } from './platform';

function getApiBase(): string {
	if (!browser) return '';

	const override = localStorage.getItem('api-server-url');
	if (override) return override;

	// Desktop Tauri: node runs on same machine
	// Mobile Tauri: node is on the network — setup flow provides the URL
	if (isTauri && !isMobile) return 'http://127.0.0.1:1530';

	return '';
}

export let apiBase = getApiBase();

export function setApiBase(url: string): void {
	apiBase = url;
}

export function apiUrl(path: string): string {
	return `${apiBase}${path}`;
}
