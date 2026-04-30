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

// Resolve the core for a specific file. The filename's extension is the
// only signal we trust — falling back to the console name would happily
// map e.g. a `.7z` archive on a Game Boy Color firkin to gambatte, which
// can't read the archive bytes and just drops the user on RetroArch's
// empty front-end. If the extension isn't a known ROM extension, return
// null so the caller hides the play affordance.
export function coreForRom(
	_consoleName: string | null | undefined,
	filename: string | null | undefined
): EmulatorCore | null {
	return coreForRomFilename(filename);
}

const ARCHIVE_EXTS: ReadonlySet<string> = new Set(['zip', '7z']);

export function isRomArchive(filename: string | null | undefined): boolean {
	if (!filename) return false;
	const idx = filename.lastIndexOf('.');
	if (idx < 0) return false;
	const ext = filename.slice(idx + 1).trim().toLowerCase();
	return ARCHIVE_EXTS.has(ext);
}

export type FirkinFileLike = {
	type: string;
	value: string;
	title?: string | null;
};

export type ResolvedRom = {
	core: EmulatorCore;
	cid: string;
	title: string;
};

// `rom_extract.rs` writes extracted files alongside their archive at
// `<archive-stem>.extracted/...` and re-pins each one. So an archive at
// title `roms/Foo.7z` is paired with one or more `ipfs` entries whose
// titles start with `roms/Foo.extracted/`. Pick the first that has a
// known ROM extension.
export function findExtractedRomFor(
	archiveTitle: string,
	files: readonly FirkinFileLike[]
): ResolvedRom | null {
	const dotIdx = archiveTitle.lastIndexOf('.');
	const stem = dotIdx < 0 ? archiveTitle : archiveTitle.slice(0, dotIdx);
	const prefix = `${stem}.extracted/`;
	for (const f of files) {
		if (f.type !== 'ipfs' || !f.title) continue;
		if (!f.title.startsWith(prefix)) continue;
		const core = coreForRomFilename(f.title);
		if (core) return { core, cid: f.value, title: f.title };
	}
	return null;
}

// Decide what (if anything) plays when a row in the firkin's file list
// is clicked. Direct ROM files resolve to themselves; archives resolve
// to whichever extracted ROM `rom_extract` produced for them. Anything
// else returns null and the row stays inert.
export function resolveRomForFile(
	file: FirkinFileLike,
	files: readonly FirkinFileLike[]
): ResolvedRom | null {
	if (file.type !== 'ipfs') return null;
	const direct = coreForRomFilename(file.title);
	if (direct) {
		return { core: direct, cid: file.value, title: file.title ?? '' };
	}
	if (isRomArchive(file.title) && file.title) {
		return findExtractedRomFor(file.title, files);
	}
	return null;
}
