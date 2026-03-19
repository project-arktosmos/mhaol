import { ObjectServiceClass } from '$services/classes/object-service.class';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

interface ThemeSettings {
	id: string;
	theme: Theme;
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
