import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

export default defineConfig({
	plugins: [svelte({ hot: !process.env.VITEST })],
	test: {
		globals: true,
		environment: 'happy-dom',
		setupFiles: ['./test/setup.ts'],
		include: ['test/**/*.{test,spec}.{js,ts}'],
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html'],
			exclude: ['node_modules/', 'test/', '**/*.d.ts', '**/*.config.*', '**/mockData', 'dist/']
		}
	},
	resolve: {
		conditions: ['browser', 'svelte'],
		alias: {
			$lib: resolve(__dirname, './src/lib'),
			$components: resolve(__dirname, './src/components'),
			'ui-lib': resolve(__dirname, './src'),
			$services: resolve(__dirname, './src/services'),
			$adapters: resolve(__dirname, './src/adapters'),
			$types: resolve(__dirname, './src/types'),
			$utils: resolve(__dirname, './src/utils'),
			$data: resolve(__dirname, './src/data'),
			$app: resolve(__dirname, './test/mocks/$app'),
			'addons/tmdb/types': resolve(__dirname, '../addons/tmdb/src/types.ts'),
			'addons/tmdb/transform': resolve(__dirname, '../addons/tmdb/src/transform.ts'),
			'addons/tmdb': resolve(__dirname, '../addons/tmdb/src/index.ts'),
			'addons/torrent-search-thepiratebay/types': resolve(
				__dirname,
				'../addons/torrent-search-thepiratebay/src/types.ts'
			),
			'addons/torrent-search-thepiratebay/format': resolve(
				__dirname,
				'../addons/torrent-search-thepiratebay/src/format.ts'
			),
			'addons/torrent-search-thepiratebay/parse-torrent-name': resolve(
				__dirname,
				'../addons/torrent-search-thepiratebay/src/parse-torrent-name.ts'
			),
			'addons/torrent-search-thepiratebay': resolve(
				__dirname,
				'../addons/torrent-search-thepiratebay/src/index.ts'
			),
			'addons/musicbrainz/types': resolve(__dirname, '../addons/musicbrainz/src/types.ts'),
			'addons/musicbrainz/transform': resolve(__dirname, '../addons/musicbrainz/src/transform.ts'),
			'addons/musicbrainz/client': resolve(__dirname, '../addons/musicbrainz/src/client.ts'),
			'addons/musicbrainz': resolve(__dirname, '../addons/musicbrainz/src/index.ts'),
			'addons/retroachievements/types': resolve(
				__dirname,
				'../addons/retroachievements/src/types.ts'
			),
			'addons/retroachievements/transform': resolve(
				__dirname,
				'../addons/retroachievements/src/transform.ts'
			),
			'addons/retroachievements': resolve(__dirname, '../addons/retroachievements/src/index.ts'),
			'addons/youtube/types': resolve(__dirname, '../addons/youtube/src/types.ts'),
			'addons/youtube/helpers': resolve(__dirname, '../addons/youtube/src/helpers.ts'),
			'addons/youtube': resolve(__dirname, '../addons/youtube/src/index.ts'),
			'addons/lrclib/types': resolve(__dirname, '../addons/lrclib/src/types.ts'),
			'addons/lrclib': resolve(__dirname, '../addons/lrclib/src/index.ts'),
			'addons/openlibrary/types': resolve(__dirname, '../addons/openlibrary/src/types.ts'),
			'addons/openlibrary/transform': resolve(__dirname, '../addons/openlibrary/src/transform.ts'),
			'addons/openlibrary': resolve(__dirname, '../addons/openlibrary/src/index.ts'),
			'torrent-search-thepiratebay': resolve(
				__dirname,
				'../addons/torrent-search-thepiratebay/src/index.ts'
			),
			webrtc: resolve(__dirname, '../webrtc/src')
		}
	}
});
