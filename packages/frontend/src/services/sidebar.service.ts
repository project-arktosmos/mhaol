import { ObjectServiceClass } from 'frontend/services/classes/object-service.class';
import type { SidebarSettings, SidebarWidthMode } from 'frontend/types/sidebar.type';

const initialSettings: SidebarSettings = {
	id: 'sidebar-settings',
	widthMode: 'default'
};

class SidebarService extends ObjectServiceClass<SidebarSettings> {
	constructor() {
		super('sidebar-settings', initialSettings);
	}

	setWidthMode(mode: SidebarWidthMode): void {
		const current = this.get();
		this.set({ ...current, widthMode: mode });
	}
}

export const sidebarService = new SidebarService();
