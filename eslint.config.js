import config from '@arktosmos/eslint-config/svelte';
import { includeIgnoreFile } from '@eslint/compat';
import { fileURLToPath } from 'node:url';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

const gitignorePath = fileURLToPath(new URL('./.gitignore', import.meta.url));

export default ts.config(includeIgnoreFile(gitignorePath), ...config, {
	files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
	languageOptions: {
		parserOptions: {
			svelteConfig
		}
	}
});
