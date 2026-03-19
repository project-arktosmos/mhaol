export type NavbarModalId =
	| 'torrent'
	| 'downloads'
	| 'jackett'
	| 'signaling'
	| 'peer-libraries'
	| 'identity'
	| 'plugins'
	| 'addons'
	| 'settings'
	| 'libraries'
	| 'llm';

export interface ModalRouterState {
	navbarModal: NavbarModalId | null;
	mediaDetail: { type: string; category: string; id: string } | null;
}
