import { ObjectServiceClass } from 'frontend/services/classes/object-service.class';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

interface ThemeSettings {
	id: string;
	theme: Theme;
}

const STORAGE_KEY = 'object-service:theme-settings';

function getBrowserPreference(): Theme {
	if (browser && window.matchMedia('(prefers-color-scheme: dark)').matches) {
		return 'dark';
	}
	return 'light';
}

const initialSettings: ThemeSettings = {
	id: 'theme-settings',
	theme: 'light'
};

class ThemeService extends ObjectServiceClass<ThemeSettings> {
	constructor() {
		super('theme-settings', initialSettings);
	}

	initialize(): void {
		if (!browser) return;
		const hasStoredPreference = localStorage.getItem(STORAGE_KEY) !== null;
		if (!hasStoredPreference) {
			this.set({ id: 'theme-settings', theme: getBrowserPreference() });
		}
		const current = this.get();
		document.documentElement.setAttribute('data-theme', current.theme);
		this.store.subscribe((settings) => {
			document.documentElement.setAttribute('data-theme', settings.theme);
		});
	}

	toggle(): void {
		const current = this.get();
		this.set({ ...current, theme: current.theme === 'light' ? 'dark' : 'light' });
	}
}

export const themeService = new ThemeService();
