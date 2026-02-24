import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({ out: 'dist' }),
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
