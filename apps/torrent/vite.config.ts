import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), '');
	const port = parseInt(env.PORT || '1520');

	return {
		plugins: [tailwindcss(), sveltekit()],
		server: {
			host: true,
			port,
			strictPort: true,
			fs: {
				allow: ['../..']
			}
		},
		preview: {
			host: true,
			port,
			strictPort: true
		}
	};
});
