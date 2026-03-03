export type NavbarModalId =
	| 'youtube'
	| 'torrent'
	| 'downloads'
	| 'libraries'
	| 'signaling'
	| 'identity'
	| 'plugins'
	| 'addons'
	| 'settings';

export interface ModalRouterState {
	navbarModal: NavbarModalId | null;
	mediaDetail: { type: string; category: string; id: string } | null;
}
