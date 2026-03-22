import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			pages: 'dist',
			assets: 'dist',
			fallback: 'index.html',
			precompress: false,
			strict: false
		}),
		paths: {
			base: '/mhaol'
		},
		alias: {
			$components: '../../packages/ui-lib/src/components',
			$utils: '../../packages/ui-lib/src/utils',
			$types: '../../packages/ui-lib/src/types',
			$data: '../../packages/ui-lib/src/data',
			$adapters: '../../packages/ui-lib/src/adapters',
			$services: '../../packages/ui-lib/src/services',
			'ui-lib': '../../packages/ui-lib/src'
		}
	}
};

export default config;
