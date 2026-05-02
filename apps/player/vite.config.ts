import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		host: true,
		port: parseInt(process.env.PORT || '9797'),
		fs: {
			allow: ['../../..']
		},
		proxy: {
			'/api': {
				target: process.env.PLAYER_API_TARGET || 'http://localhost:9898',
				changeOrigin: true
			}
		}
	},
	preview: {
		host: true,
		port: parseInt(process.env.PORT || '9797')
	},
	define: {
		// Helia / libp2p occasionally probe `globalThis.process`; provide a
		// minimal shim for the browser bundle so dependencies that read
		// `process.env.NODE_DEBUG` etc. don't crash.
		'process.env.NODE_DEBUG': 'undefined'
	},
	optimizeDeps: {
		// Pre-bundle the heaviest libp2p / helia entry points so the dev
		// server doesn't have to walk their dependency graphs on cold reload.
		include: [
			'helia',
			'@helia/unixfs',
			'libp2p',
			'@libp2p/websockets',
			'@libp2p/webtransport',
			'@libp2p/bootstrap',
			'@libp2p/identify',
			'@libp2p/pnet',
			'@chainsafe/libp2p-noise',
			'@chainsafe/libp2p-yamux',
			'@multiformats/multiaddr',
			'multiformats/cid',
			'uint8arrays/concat',
			'uint8arrays/to-string'
		]
	}
});
