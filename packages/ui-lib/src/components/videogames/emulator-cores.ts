// Mapping from console / ROM extension to the EmulatorJS core name.
// EmulatorJS core ids documented at https://emulatorjs.org/docs4devs/cores.
//
// Only Game Boy Color is wired up for now (gambatte) — the other entries
// are scaffolding so the modal can be reused as more consoles are tested.

export type EmulatorCore =
	| 'gambatte' // Game Boy / Game Boy Color
	| 'mgba' // Game Boy Advance
	| 'fceumm' // NES / Famicom
	| 'snes9x' // SNES / Super Famicom
	| 'mupen64plus_next' // Nintendo 64
	| 'genesis_plus_gx' // Mega Drive / Master System / Game Gear
	| 'melonds' // Nintendo DS
	| 'pcsx_rearmed'; // PlayStation

const CONSOLE_NAME_TO_CORE: Record<string, EmulatorCore> = {
	'game boy': 'gambatte',
	'game boy color': 'gambatte'
};

const EXTENSION_TO_CORE: Record<string, EmulatorCore> = {
	gb: 'gambatte',
	gbc: 'gambatte'
};

export function coreForConsoleName(name: string | null | undefined): EmulatorCore | null {
	if (!name) return null;
	return CONSOLE_NAME_TO_CORE[name.trim().toLowerCase()] ?? null;
}

export function coreForRomFilename(filename: string | null | undefined): EmulatorCore | null {
	if (!filename) return null;
	const idx = filename.lastIndexOf('.');
	if (idx < 0) return null;
	const ext = filename.slice(idx + 1).trim().toLowerCase();
	if (!ext) return null;
	return EXTENSION_TO_CORE[ext] ?? null;
}

export function coreForRom(
	consoleName: string | null | undefined,
	filename: string | null | undefined
): EmulatorCore | null {
	return coreForRomFilename(filename) ?? coreForConsoleName(consoleName);
}
