import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { routeDiscovery } from './vite-plugin-routes.js';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), routeDiscovery()],
	server: {
		port: 1530
	}
});
