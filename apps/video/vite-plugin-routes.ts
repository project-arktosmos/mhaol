import { readdirSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import type { Plugin, ViteDevServer } from 'vite';
import type { RouteEntry } from './src/types/route.type.js';

const VIRTUAL_MODULE_ID = 'virtual:routes';
const RESOLVED_VIRTUAL_MODULE_ID = '\0' + VIRTUAL_MODULE_ID;

function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1);
}

function scanRoutes(dir: string, basePath: string = ''): RouteEntry[] {
	const entries: RouteEntry[] = [];

	let items;
	try {
		items = readdirSync(dir, { withFileTypes: true });
	} catch {
		return entries;
	}

	for (const item of items) {
		if (!item.isDirectory()) continue;
		if (item.name === 'api') continue;
		if (item.name.startsWith('(')) continue;

		const fullPath = join(dir, item.name);
		const routePath = basePath + '/' + item.name;
		const hasPage = existsSync(join(fullPath, '+page.svelte'));
		const isDynamic = /^\[.+\]$/.test(item.name);
		const children = scanRoutes(fullPath, routePath);

		if (hasPage || children.length > 0) {
			entries.push({
				path: routePath,
				label: isDynamic ? item.name : capitalize(item.name),
				isDynamic,
				children
			});
		}
	}

	return entries;
}

export function routeDiscovery(): Plugin {
	let routesDir: string;
	let server: ViteDevServer | undefined;

	function generateRouteData(): RouteEntry[] {
		const routes: RouteEntry[] = [];

		if (existsSync(join(routesDir, '+page.svelte'))) {
			routes.push({
				path: '/',
				label: 'Home',
				isDynamic: false,
				children: []
			});
		}

		routes.push(...scanRoutes(routesDir));
		return routes;
	}

	function invalidateVirtualModule() {
		if (!server) return;
		const mod = server.moduleGraph.getModuleById(RESOLVED_VIRTUAL_MODULE_ID);
		if (mod) {
			server.moduleGraph.invalidateModule(mod);
			server.hot.send({ type: 'full-reload' });
		}
	}

	return {
		name: 'route-discovery',

		configResolved(config) {
			routesDir = resolve(config.root, 'src/routes');
		},

		resolveId(id) {
			if (id === VIRTUAL_MODULE_ID) {
				return RESOLVED_VIRTUAL_MODULE_ID;
			}
		},

		load(id) {
			if (id === RESOLVED_VIRTUAL_MODULE_ID) {
				const routes = generateRouteData();
				return `export default ${JSON.stringify(routes)};`;
			}
		},

		configureServer(devServer) {
			server = devServer;

			server.watcher.on('add', (filePath: string) => {
				if (filePath.endsWith('+page.svelte') && filePath.includes('/routes/')) {
					invalidateVirtualModule();
				}
			});

			server.watcher.on('unlink', (filePath: string) => {
				if (filePath.endsWith('+page.svelte') && filePath.includes('/routes/')) {
					invalidateVirtualModule();
				}
			});
		}
	};
}
