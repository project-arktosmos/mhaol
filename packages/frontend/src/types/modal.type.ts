export type NavbarModalId =
	| 'youtube'
	| 'youtube-search'
	| 'torrent'
	| 'downloads'
	| 'jackett'
	| 'libraries'
	| 'signaling'
	| 'peer-libraries'
	| 'identity'
	| 'plugins'
	| 'addons'
	| 'settings'
	| 'llm';

export interface ModalRouterState {
	navbarModal: NavbarModalId | null;
	mediaDetail: { type: string; category: string; id: string } | null;
}
