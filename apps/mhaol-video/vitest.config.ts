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
		alias: {
			$lib: resolve(__dirname, './src/lib'),
			$services: resolve(__dirname, './src/services'),
			$adapters: resolve(__dirname, './src/adapters'),
			$types: resolve(__dirname, './src/types'),
			$utils: resolve(__dirname, './src/utils'),
			$app: resolve(__dirname, './test/mocks/$app')
		}
	}
});
