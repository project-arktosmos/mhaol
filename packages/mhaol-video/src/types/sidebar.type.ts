import type { ID } from '$types/core.type';

export type SidebarWidthMode = 'wide' | 'default' | 'narrow';

export interface SidebarSettings {
	id: ID;
	widthMode: SidebarWidthMode;
}
