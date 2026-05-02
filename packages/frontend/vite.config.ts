import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { navPlugin } from './scripts/nav-vite-plugin.mjs';

export default defineConfig({
	plugins: [navPlugin(), tailwindcss(), sveltekit()],
	server: {
		host: true,
		port: parseInt(process.env.PORT || '9898'),
		fs: {
			allow: ['../..']
		},
		proxy: {
			'/api': 'http://127.0.0.1:9899'
		}
	},
	preview: {
		host: true,
		port: parseInt(process.env.PORT || '9898'),
		proxy: {
			'/api': 'http://127.0.0.1:9899'
		}
	}
});
