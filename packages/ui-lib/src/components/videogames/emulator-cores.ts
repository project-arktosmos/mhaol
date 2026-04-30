// Mapping from console / ROM extension to the EmulatorJS core name.
// EmulatorJS core ids documented at https://emulatorjs.org/docs4devs/cores.
// Status per console is tracked in `addons/retroachievements/types`'s
// `CONSOLE_WASM_STATUS`; see `docs/wasm-emulators.md` for the research.

export type EmulatorCore =
	| 'gambatte' // Game Boy / Game Boy Color
	| 'mgba' // Game Boy Advance
	| 'fceumm' // NES / Famicom
	| 'snes9x' // SNES / Super Famicom
	| 'mupen64plus_next' // Nintendo 64
	| 'genesis_plus_gx' // Mega Drive / Master System / Game Gear
	| 'melonds' // Nintendo DS
	| 'pcsx_rearmed' // PlayStation
	| 'stella2014' // Atari 2600
	| 'prosystem' // Atari 7800
	| 'handy' // Atari Lynx
	| 'mednafen_pce' // PC Engine / TurboGrafx-16
	| 'mednafen_ngp' // Neo Geo Pocket
	| 'virtualjaguar'; // Atari Jaguar

const CONSOLE_NAME_TO_CORE: Record<string, EmulatorCore> = {
	'game boy': 'gambatte',
	'game boy color': 'gambatte',
	'game boy advance': 'mgba',
	'nes/famicom': 'fceumm',
	'snes/super famicom': 'snes9x',
	'nintendo 64': 'mupen64plus_next',
	'genesis/mega drive': 'genesis_plus_gx',
	'master system': 'genesis_plus_gx',
	'nintendo ds': 'melonds',
	playstation: 'pcsx_rearmed',
	'atari 2600': 'stella2014',
	'atari 7800': 'prosystem',
	'atari lynx': 'handy',
	'pc engine/turbografx-16': 'mednafen_pce',
	'neo geo pocket': 'mednafen_ngp',
	'atari jaguar': 'virtualjaguar'
};

const EXTENSION_TO_CORE: Record<string, EmulatorCore> = {
	gb: 'gambatte',
	gbc: 'gambatte',
	gba: 'mgba',
	nes: 'fceumm',
	smc: 'snes9x',
	sfc: 'snes9x',
	n64: 'mupen64plus_next',
	z64: 'mupen64plus_next',
	v64: 'mupen64plus_next',
	md: 'genesis_plus_gx',
	gen: 'genesis_plus_gx',
	smd: 'genesis_plus_gx',
	sms: 'genesis_plus_gx',
	gg: 'genesis_plus_gx',
	nds: 'melonds',
	// PS1 disc images. `.bin` is intentionally omitted because it overlaps
	// with raw Genesis dumps and naked PS1 `.bin` files (without their
	// `.cue` sheet) won't load anyway.
	cue: 'pcsx_rearmed',
	chd: 'pcsx_rearmed',
	pbp: 'pcsx_rearmed',
	iso: 'pcsx_rearmed',
	a26: 'stella2014',
	a78: 'prosystem',
	lnx: 'handy',
	lyx: 'handy',
	pce: 'mednafen_pce',
	ngp: 'mednafen_ngp',
	ngc: 'mednafen_ngp',
	j64: 'virtualjaguar',
	jag: 'virtualjaguar'
};

export function coreForConsoleName(name: string | null | undefined): EmulatorCore | null {
	if (!name) return null;
	return CONSOLE_NAME_TO_CORE[name.trim().toLowerCase()] ?? null;
}

export function coreForRomFilename(filename: string | null | undefined): EmulatorCore | null {
	if (!filename) return null;
	const idx = filename.lastIndexOf('.');
	if (idx < 0) return null;
	const ext = filename
		.slice(idx + 1)
		.trim()
		.toLowerCase();
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
	const ext = filename
		.slice(idx + 1)
		.trim()
		.toLowerCase();
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
