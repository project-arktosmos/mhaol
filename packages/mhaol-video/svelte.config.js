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
			$components: 'src/components/*',
			$utils: 'src/utils/*',
			$types: 'src/types/*',
			$data: 'src/data/*',
			$adapters: 'src/adapters/*',
			$services: 'src/services/*'
		}
	}
};

export default config;
