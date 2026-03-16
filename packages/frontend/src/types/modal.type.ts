export type NavbarModalId =
	| 'torrent'
	| 'downloads'
	| 'signaling'
	| 'identity'
	| 'plugins'
	| 'addons'
	| 'settings';

export interface ModalRouterState {
	navbarModal: NavbarModalId | null;
	mediaDetail: { type: string; category: string; id: string } | null;
}
