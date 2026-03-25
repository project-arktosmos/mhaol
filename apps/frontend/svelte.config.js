import adapterStatic from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit') .Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		appDir: 'app',
		paths: {
			base: process.env.BASE_PATH || ''
		},
		adapter: adapterStatic({
			fallback: 'index.html',
			pages: 'dist-static',
			assets: 'dist-static'
		}),
		alias: {
			$components: '../../packages/ui-lib/src/components',
			$utils: '../../packages/ui-lib/src/utils',
			$types: '../../packages/ui-lib/src/types',
			$data: '../../packages/ui-lib/src/data',
			$adapters: '../../packages/ui-lib/src/adapters',
			$services: '../../packages/ui-lib/src/services',
			'ui-lib': '../../packages/ui-lib/src',
			webrtc: '../../packages/webrtc/src',
			'addons/tmdb': '../../packages/addons/tmdb/src',
			'addons/torrent-search-thepiratebay': '../../packages/addons/torrent-search-thepiratebay/src',
			'addons/musicbrainz': '../../packages/addons/musicbrainz/src',
			'addons/retroachievements': '../../packages/addons/retroachievements/src',
			'addons/youtube': '../../packages/addons/youtube/src',
			'addons/lrclib': '../../packages/addons/lrclib/src',
			'addons/openlibrary': '../../packages/addons/openlibrary/src',
			'assets/game-consoles': '../../packages/assets/game-consoles',
			'torrent-search-thepiratebay': '../../packages/addons/torrent-search-thepiratebay/src/index.ts'
		}
	}
};

export default config;
