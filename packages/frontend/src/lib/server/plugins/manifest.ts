import type { PluginManifest } from './types';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function extractManifest(pkg: Record<string, any>): PluginManifest {
	const addon = pkg.addon ?? {};
	return {
		name: pkg.name,
		version: pkg.version,
		description: pkg.description ?? '',
		source: addon.source,
		compatibility: addon.compatibility,
		schema: addon.schema,
		settings: addon.settings
	};
}
