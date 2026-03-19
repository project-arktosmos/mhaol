import type { StorybookConfig } from '@storybook/svelte-vite';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const __dirname = dirname(fileURLToPath(import.meta.url));

const config: StorybookConfig = {
	stories: ['../src/**/*.stories.ts'],
	addons: [],
	framework: '@storybook/svelte-vite',
	typescript: {
		check: false
	},
	async viteFinal(config) {
		const { mergeConfig } = await import('vite');
		const tailwindcss = (await import('@tailwindcss/vite')).default;

		// Remove the default svelte plugin and docgen plugin, replace with one that handles node_modules
		if (config.plugins) {
			config.plugins = (config.plugins as Array<{ name?: string } | Array<{ name?: string }>>)
				.flat()
				.filter((p) => {
					if (p && typeof p === 'object' && 'name' in p) {
						return (
							p.name !== 'storybook:svelte-docgen-plugin' &&
							p.name !== 'vite-plugin-svelte'
						);
					}
					return true;
				});
		}

		return mergeConfig(config, {
			plugins: [
				tailwindcss(),
				svelte({
					// Include @storybook svelte files from node_modules
					configFile: resolve(__dirname, '../svelte.config.js')
				})
			],
			resolve: {
				alias: {
					$components: resolve(__dirname, '../../../packages/ui-lib/src/components'),
					$utils: resolve(__dirname, '../../../packages/frontend/src/utils'),
					$types: resolve(__dirname, '../../../packages/frontend/src/types'),
					$data: resolve(__dirname, '../../../packages/frontend/src/data'),
					$adapters: resolve(__dirname, '../../../packages/frontend/src/adapters'),
					$services: resolve(__dirname, '../../../packages/frontend/src/services'),
					frontend: resolve(__dirname, '../../../packages/frontend/src'),
					'ui-lib': resolve(__dirname, '../../../packages/ui-lib/src'),
					'$app/environment': resolve(__dirname, '../src/mocks/app-environment.ts'),
					'$app/paths': resolve(__dirname, '../src/mocks/app-paths.ts'),
					'addons/tmdb/types': resolve(__dirname, '../../../packages/addons/tmdb/src/types.ts'),
					'addons/tmdb/transform': resolve(__dirname, '../../../packages/addons/tmdb/src/transform.ts'),
					'addons/tmdb': resolve(__dirname, '../../../packages/addons/tmdb/src/index.ts'),
					'addons/torrent-search-thepiratebay/types': resolve(
						__dirname,
						'../../../packages/addons/torrent-search-thepiratebay/src/types.ts'
					),
					'addons/torrent-search-thepiratebay': resolve(
						__dirname,
						'../../../packages/addons/torrent-search-thepiratebay/src/index.ts'
					),
					'torrent-search-thepiratebay': resolve(
						__dirname,
						'../../../packages/addons/torrent-search-thepiratebay/src/index.ts'
					)
				}
			},
			optimizeDeps: {
				// Exclude @storybook/svelte from optimization so svelte files get compiled
				exclude: ['@storybook/svelte']
			}
		});
	}
};

export default config;
