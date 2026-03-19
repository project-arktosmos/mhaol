declare module 'virtual:routes' {
	interface RouteEntry {
		path: string;
		label: string;
		isDynamic: boolean;
		children: RouteEntry[];
	}

	const routes: RouteEntry[];
	export default routes;
}
