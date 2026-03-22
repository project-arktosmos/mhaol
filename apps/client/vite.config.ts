import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const port = Number(process.env.PORT) || 1570;

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		host: true,
		port,
		fs: {
			allow: ['../..']
		},
		proxy: {
			'/api': `http://localhost:${port}`
		}
	},
	preview: {
		host: true,
		port,
		proxy: {
			'/api': `http://localhost:${port}`
		}
	}
});
