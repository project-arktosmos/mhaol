import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), '');
	const port = parseInt(env.PORT || '1530');

	return {
		plugins: [tailwindcss(), sveltekit()],
		server: {
			host: true,
			port,
			strictPort: true,
			fs: {
				allow: ['../..']
			},
			proxy: {
				'/api': `http://localhost:${port - 1}`
			}
		},
		preview: {
			host: true,
			port,
			strictPort: true,
			proxy: {
				'/api': `http://localhost:${port - 1}`
			}
		}
	};
});
