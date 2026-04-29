import adapterStatic from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
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
			$components: '../../../packages/ui-lib/src/components',
			$utils: '../../../packages/ui-lib/src/utils',
			$types: '../../../packages/ui-lib/src/types',
			$data: '../../../packages/ui-lib/src/data',
			$adapters: '../../../packages/ui-lib/src/adapters',
			$services: '../../../packages/ui-lib/src/services',
			'ui-lib': '../../../packages/ui-lib/src'
		}
	}
};

export default config;
