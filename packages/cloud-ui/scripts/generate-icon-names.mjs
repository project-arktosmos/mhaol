#!/usr/bin/env node
import { readdirSync, statSync, writeFileSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
const assetsDir = join(here, '..', 'src', 'icons', 'assets');
const outFile = join(here, '..', 'src', 'icons', 'icon-names.ts');

function walk(dir) {
	const out = [];
	for (const entry of readdirSync(dir)) {
		const full = join(dir, entry);
		const st = statSync(full);
		if (st.isDirectory()) out.push(...walk(full));
		else if (entry.endsWith('.svg')) out.push(full);
	}
	return out;
}

const names = walk(assetsDir)
	.map((p) => relative(assetsDir, p).replace(/\\/g, '/').replace(/\.svg$/, ''))
	.sort();

const lines = [
	'// Generated from src/icons/assets/<author>/<name>.svg — do not hand-edit.',
	'// Re-run scripts/generate-icon-names.mjs to refresh.',
	'',
	'export const ICON_NAMES = ['
];
for (const n of names) lines.push(`\t${JSON.stringify(n)},`);
lines.push('] as const;');
lines.push('');
lines.push('export type IconName = (typeof ICON_NAMES)[number];');
lines.push('');

writeFileSync(outFile, lines.join('\n'));
console.log(`Wrote ${names.length} icons → ${relative(process.cwd(), outFile)}`);
