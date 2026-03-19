import adapterStatic from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit') .Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapterStatic({
			fallback: 'index.html',
			pages: 'dist-static',
			assets: 'dist-static'
		}),
		alias: {
			$components: '../../packages/frontend/src/components',
			$utils: '../../packages/frontend/src/utils',
			$types: '../../packages/frontend/src/types',
			$data: '../../packages/frontend/src/data',
			$adapters: '../../packages/frontend/src/adapters',
			$services: '../../packages/frontend/src/services',
			frontend: '../../packages/frontend/src'
		}
	}
};

export default config;
