import { ObjectServiceClass } from '$services/classes/object-service.class';

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

	toggle(): void {
		const current = this.get();
		this.set({ ...current, theme: current.theme === 'light' ? 'dark' : 'light' });
	}
}

export const themeService = new ThemeService();
