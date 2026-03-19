import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://svelte.dev/docs/kit/integrations
	// for more information about preprocessors
	preprocess: vitePreprocess(),

	kit: {
		// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://svelte.dev/docs/kit/adapters for more information about adapters.
		adapter: adapter({
			pages: 'dist',
			assets: 'dist',
			fallback: 'index.html', // This is crucial for SPA mode
			precompress: false,
			strict: false // This tells the adapter to ignore dynamic routes
		}),
		paths: {
			base: '/mhaoltube'
		},
		alias: {
			$components: '../../packages/ui-lib/src/components',
			$utils: '../../packages/frontend/src/utils',
			$types: '../../packages/frontend/src/types',
			$data: '../../packages/frontend/src/data',
			$adapters: '../../packages/frontend/src/adapters',
			$services: '../../packages/frontend/src/services',
			frontend: '../../packages/frontend/src',
			'ui-lib': '../../packages/ui-lib/src'
		}
	}
};

export default config;
