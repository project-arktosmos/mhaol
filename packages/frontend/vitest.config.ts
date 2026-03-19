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
		conditions: ['svelte'],
		alias: {
			$lib: resolve(__dirname, './src/lib'),
			$components: resolve(__dirname, '../ui-lib/src/components'),
			'ui-lib': resolve(__dirname, '../ui-lib/src'),
			$services: resolve(__dirname, './src/services'),
			$adapters: resolve(__dirname, './src/adapters'),
			$types: resolve(__dirname, './src/types'),
			$utils: resolve(__dirname, './src/utils'),
			$data: resolve(__dirname, './src/data'),
			$app: resolve(__dirname, './test/mocks/$app'),
			frontend: resolve(__dirname, './src')
		}
	}
});
