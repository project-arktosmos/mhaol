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
			$components: 'src/components',
			$ipfs: 'src/ipfs',
			$types: 'src/types',
			$utils: 'src/utils'
		}
	}
};

export default config;
