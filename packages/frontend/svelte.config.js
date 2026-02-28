import adapterNode from '@sveltejs/adapter-node';
import adapterStatic from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const isMobile =
	process.env.TAURI_ENV_PLATFORM === 'android' || process.env.BUILD_ADAPTER === 'static';

/** @type {import('@sveltejs/kit') .Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: isMobile
			? adapterStatic({ fallback: 'index.html', pages: 'dist-static', assets: 'dist-static' })
			: adapterNode({ out: 'dist' }),
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
