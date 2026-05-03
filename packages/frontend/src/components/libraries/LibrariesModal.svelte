<script lang="ts">
	import { onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import Modal from '$components/core/Modal.svelte';
	import { librariesModalService } from '$services/libraries-modal.service';
	import {
		librariesService,
		LIBRARY_ADDONS,
		LIBRARY_ADDON_LABELS,
		type LibraryAddon,
		type ScanResponse,
		type ScanEntry,
		type Library
	} from '$lib/libraries.service';

	type PreflightState =
		| { status: 'typing' | 'searching' | 'no_match' }
		| {
				status: 'found';
				tmdbId: string;
				tmdbTitle: string;
				tmdbYear?: number;
				seasonCount?: number;
		  }
		| { status: 'error'; error: string };
	import type { IpfsPin } from '$lib/ipfs.service';
	import { firkinsService, type Artist, type Trailer, type Review } from '$lib/firkins.service';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import DirectoryPicker from '$components/DirectoryPicker.svelte';

	const libsStore = librariesService.state;
	const modalStore = librariesModalService.store;
	const SCAN_STALE_MS = 60 * 60 * 1000;
	let firstOpenSeen = false;

	function close() {
		librariesModalService.close();
	}

	let pickedDir = $state('');
	let newSubfolder = $state('');
	let newAddons = $state<LibraryAddon[]>([]);
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let deletingId = $state<string | null>(null);
	let scanningId = $state<string | null>(null);
	let scanResults = $state<Record<string, ScanResponse>>({});
	let scanErrors = $state<Record<string, string>>({});
	let libPins = $state<Record<string, IpfsPin[]>>({});
	let pinsErrors = $state<Record<string, string>>({});
	let editingKindsFor = $state<string | null>(null);
	let editAddons = $state<LibraryAddon[]>([]);
	let savingKinds = $state(false);
	let addonsError = $state<string | null>(null);
	let creatingFirkinFor = $state<Record<string, boolean>>({});
	let createFirkinErrors = $state<Record<string, string>>({});

	/// User-edited TV show name per group, keyed by the group's *original*
	/// extracted show key (`showKey(libId, originalGroup)`). Defaulted to the
	/// extracted show name on first sight of each scan group, then mutated
	/// in-place by the per-group input. The effective name fed to the
	/// preflight + the build is `editedShowNames[origKey] ?? group.show`.
	let editedShowNames = $state<Record<string, string>>({});
	let preflightStates = $state<Record<string, PreflightState>>({});
	const preflightTimers = new Map<string, number>();
	const preflightAborters = new Map<string, AbortController>();

	function toggleNewAddon(addon: LibraryAddon) {
		newAddons = newAddons.includes(addon)
			? newAddons.filter((k) => k !== addon)
			: [...newAddons, addon];
	}

	function toggleEditAddon(addon: LibraryAddon) {
		editAddons = editAddons.includes(addon)
			? editAddons.filter((k) => k !== addon)
			: [...editAddons, addon];
	}

	function startEditKinds(lib: Library) {
		editingKindsFor = lib.id;
		editAddons = [...(lib.addons ?? [])];
		addonsError = null;
	}

	function cancelEditKinds() {
		editingKindsFor = null;
		editAddons = [];
		addonsError = null;
	}

	async function saveKinds(id: string) {
		savingKinds = true;
		addonsError = null;
		try {
			await librariesService.update(id, { addons: editAddons });
			editingKindsFor = null;
			editAddons = [];
		} catch (err) {
			addonsError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			savingKinds = false;
		}
	}

	$effect(() => {
		if (!$modalStore.open || firstOpenSeen) return;
		firstOpenSeen = true;
		void (async () => {
			await librariesService.refresh();
			const { libraries } = get(libsStore);
			await Promise.all(libraries.map(handleLibraryOnMount));
			// Re-hydrate the in-progress badge for any TV firkin builds the
			// server is still running. Polling kicks in only when at least one
			// non-terminal job exists; the loop self-stops once everything is
			// either completed or errored.
			await Promise.all(libraries.map(hydrateTvBuildsForLibrary));
		})();
	});

	onDestroy(() => {
		for (const handle of tvBuildPollers.values()) {
			window.clearInterval(handle);
		}
		tvBuildPollers.clear();
		for (const handle of preflightTimers.values()) {
			window.clearTimeout(handle);
		}
		preflightTimers.clear();
		for (const ac of preflightAborters.values()) {
			ac.abort();
		}
		preflightAborters.clear();
	});

	async function runPreflight(
		libId: string,
		origKey: string,
		show: string,
		year: number | undefined
	) {
		const trimmed = show.trim();
		if (!trimmed) {
			preflightStates = { ...preflightStates, [origKey]: { status: 'no_match' } };
			return;
		}
		const prev = preflightAborters.get(origKey);
		if (prev) prev.abort();
		const ac = new AbortController();
		preflightAborters.set(origKey, ac);
		preflightStates = { ...preflightStates, [origKey]: { status: 'searching' } };
		try {
			const res = await librariesService.tvPreflight(libId, trimmed, year, ac.signal);
			if (ac.signal.aborted) return;
			if (!res.tmdbId) {
				preflightStates = {
					...preflightStates,
					[origKey]: { status: 'no_match' }
				};
			} else {
				preflightStates = {
					...preflightStates,
					[origKey]: {
						status: 'found',
						tmdbId: res.tmdbId,
						tmdbTitle: res.tmdbTitle ?? '',
						tmdbYear: res.tmdbYear ?? undefined,
						seasonCount: res.seasonCount ?? undefined
					}
				};
			}
		} catch (err) {
			if (err instanceof DOMException && err.name === 'AbortError') return;
			const message = err instanceof Error ? err.message : 'Unknown error';
			preflightStates = {
				...preflightStates,
				[origKey]: { status: 'error', error: message }
			};
		} finally {
			if (preflightAborters.get(origKey) === ac) {
				preflightAborters.delete(origKey);
			}
		}
	}

	function onShowNameInput(
		libId: string,
		origKey: string,
		value: string,
		year: number | undefined
	) {
		editedShowNames = { ...editedShowNames, [origKey]: value };
		preflightStates = { ...preflightStates, [origKey]: { status: 'typing' } };
		const existing = preflightTimers.get(origKey);
		if (existing !== undefined) window.clearTimeout(existing);
		const handle = window.setTimeout(() => {
			preflightTimers.delete(origKey);
			void runPreflight(libId, origKey, value, year);
		}, 400);
		preflightTimers.set(origKey, handle);
	}

	function firePreflightForLibrary(libId: string, scan: ScanResponse) {
		const groups = groupEntries(scan.entries);
		for (const group of groups) {
			if (group.kind !== 'show') continue;
			const origKey = showKey(libId, group);
			if (editedShowNames[origKey] === undefined) {
				editedShowNames = { ...editedShowNames, [origKey]: group.show };
			}
			const effShow = editedShowNames[origKey] ?? group.show;
			void runPreflight(libId, origKey, effShow, group.year);
		}
	}

	async function hydrateTvBuildsForLibrary(lib: Library) {
		try {
			const snapshot = await pollTvBuilds(lib.id);
			applyTvBuildSnapshot(lib.id, snapshot);
			if (hasActiveBuild(lib.id)) {
				ensureTvBuildPolling(lib.id);
			}
		} catch {
			// non-fatal: the next user action will retry
		}
	}

	function isStale(lib: Library): boolean {
		if (!lib.last_scanned_at) return true;
		const ts = new Date(lib.last_scanned_at).getTime();
		if (Number.isNaN(ts)) return true;
		return Date.now() - ts > SCAN_STALE_MS;
	}

	async function handleLibraryOnMount(lib: Library) {
		if (isStale(lib)) {
			await scan(lib.id);
		} else {
			await Promise.all([loadPins(lib.id), loadScanResult(lib.id)]);
		}
	}

	async function loadScanResult(id: string) {
		try {
			const result = await librariesService.lastScanResult(id);
			if (result) {
				scanResults = { ...scanResults, [id]: result };
				firePreflightForLibrary(id, result);
			}
		} catch {
			// no persisted scan result yet — leave scanResults untouched
		}
	}

	async function loadPins(id: string) {
		try {
			const pins = await librariesService.pins(id);
			libPins = { ...libPins, [id]: pins };
			const { [id]: _ignored, ...rest } = pinsErrors;
			pinsErrors = rest;
		} catch (err) {
			pinsErrors = {
				...pinsErrors,
				[id]: err instanceof Error ? err.message : 'Unknown error'
			};
		}
	}

	function sanitize(value: string): string {
		// eslint-disable-next-line no-control-regex
		const controlAndIllegal = /[\\/:*?"<>|\x00-\x1f]/g;
		return value.replace(controlAndIllegal, '_').trim();
	}

	function joinPath(base: string, child: string): string {
		if (!base) return child;
		const sep = base.includes('\\') && !base.includes('/') ? '\\' : '/';
		const trimmed = base.endsWith('/') || base.endsWith('\\') ? base.slice(0, -1) : base;
		return `${trimmed}${sep}${child}`;
	}

	const finalPath = $derived.by(() => {
		const sub = sanitize(newSubfolder);
		if (!pickedDir) return '';
		if (!sub) return pickedDir;
		return joinPath(pickedDir, sub);
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		createError = null;
		if (!finalPath) {
			createError = 'Pick a directory';
			return;
		}
		creating = true;
		try {
			const created = await librariesService.create(finalPath, newAddons);
			newSubfolder = '';
			newAddons = [];
			void scan(created.id);
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			creating = false;
		}
	}

	const pinPollers = new Map<string, number>();

	function pollPinsUntilStable(id: string) {
		const existing = pinPollers.get(id);
		if (existing !== undefined) {
			window.clearInterval(existing);
		}
		let lastCount = libPins[id]?.length ?? 0;
		let stableTicks = 0;
		const handle = window.setInterval(async () => {
			await loadPins(id);
			const current = libPins[id]?.length ?? 0;
			if (current === lastCount) {
				stableTicks++;
				if (stableTicks >= 3) {
					window.clearInterval(handle);
					pinPollers.delete(id);
				}
			} else {
				stableTicks = 0;
				lastCount = current;
			}
		}, 3000);
		pinPollers.set(id, handle);
		window.setTimeout(() => {
			const h = pinPollers.get(id);
			if (h !== undefined) {
				window.clearInterval(h);
				pinPollers.delete(id);
			}
		}, 120000);
	}

	async function scan(id: string) {
		scanningId = id;
		const { [id]: _ignored, ...rest } = scanErrors;
		scanErrors = rest;
		try {
			const result = await librariesService.scan(id);
			scanResults = { ...scanResults, [id]: result };
			firePreflightForLibrary(id, result);
			await Promise.all([librariesService.refresh(), loadPins(id)]);
			pollPinsUntilStable(id);
		} catch (err) {
			scanErrors = {
				...scanErrors,
				[id]: err instanceof Error ? err.message : 'Unknown error'
			};
		} finally {
			scanningId = null;
		}
	}

	function clearScan(id: string) {
		const { [id]: _ignored, ...rest } = scanResults;
		scanResults = rest;
		const { [id]: _ignored2, ...errRest } = scanErrors;
		scanErrors = errRest;
	}

	function pinIndex(pins: IpfsPin[] | undefined): Map<string, string> {
		const map = new Map<string, string>();
		if (!pins) return map;
		for (const p of pins) map.set(p.path, p.cid);
		return map;
	}

	/// Poll the library's pins endpoint until a pin for `filePath` appears,
	/// returning its CID. The library scan's pin task runs in the
	/// background, so when the user clicks "Create firkin" right after a
	/// scan the cid for that specific file may not be recorded yet — this
	/// blocks until it is, instead of letting the firkin be created with no
	/// `ipfs` file entry. Returns `undefined` only if the timeout elapses.
	async function waitForPinForPath(
		libId: string,
		filePath: string,
		timeoutMs = 60000
	): Promise<string | undefined> {
		const start = Date.now();
		while (Date.now() - start < timeoutMs) {
			try {
				const pins = await librariesService.pins(libId);
				libPins = { ...libPins, [libId]: pins };
				const match = pins.find((p) => p.path === filePath);
				if (match) return match.cid;
			} catch {
				// ignore and retry
			}
			await new Promise((r) => setTimeout(r, 1000));
		}
		return undefined;
	}

	/// Create a `tmdb-movie` firkin from a TMDB-matched scan entry. Mirrors
	/// the /catalog/virtual bookmark flow: hit /metadata for artists +
	/// trailers + reviews, build a payload identical to what the catalog
	/// virtual page sends, then POST to /api/firkins. The only difference
	/// is `files`: in addition to the canonical `url` entry pointing at
	/// TMDB, we attach the pinned IPFS cid so the resulting firkin already
	/// has a playable file. After create, navigate to the new content-
	/// addressed detail page.
	async function createFirkinFromMatch(
		libId: string,
		entry: {
			path: string;
			relative_path: string;
			tmdbMatch?: {
				tmdbId: number;
				title: string;
				year?: number;
				overview?: string;
				posterUrl?: string;
			};
		},
		cid: string | undefined
	) {
		if (!entry.tmdbMatch) return;
		const key = entry.path;
		if (creatingFirkinFor[key]) return;
		creatingFirkinFor = { ...creatingFirkinFor, [key]: true };
		const { [key]: _ignored, ...errRest } = createFirkinErrors;
		createFirkinErrors = errRest;
		try {
			let resolvedCid = cid;
			if (!resolvedCid) {
				resolvedCid = await waitForPinForPath(libId, entry.path);
				if (!resolvedCid) {
					throw new Error(
						'IPFS pin for this file did not complete in time — try again once the CID column populates'
					);
				}
			}

			const tmdbId = String(entry.tmdbMatch.tmdbId);
			const metaRes = await fetch(
				`/api/catalog/tmdb-movie/${encodeURIComponent(tmdbId)}/metadata`,
				{ cache: 'no-store' }
			);
			if (!metaRes.ok) {
				let message = `HTTP ${metaRes.status}`;
				try {
					const body = await metaRes.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const meta = (await metaRes.json()) as {
				artists?: Artist[];
				trailers?: Trailer[];
				reviews?: Review[];
			};

			const fileTitle = entry.relative_path.split('/').pop() ?? entry.relative_path;
			const created = await firkinsService.create({
				title: entry.tmdbMatch.title,
				artists: meta.artists ?? [],
				description: entry.tmdbMatch.overview ?? '',
				images: entry.tmdbMatch.posterUrl
					? [
							{
								url: entry.tmdbMatch.posterUrl,
								mimeType: 'image/jpeg',
								fileSize: 0,
								width: 0,
								height: 0
							}
						]
					: [],
				files: [
					{
						type: 'url',
						value: `https://www.themoviedb.org/movie/${tmdbId}`,
						title: 'TMDB Movie'
					},
					{ type: 'ipfs' as const, value: resolvedCid, title: fileTitle }
				],
				year: entry.tmdbMatch.year ?? null,
				addon: 'tmdb-movie',
				trailers: meta.trailers ?? [],
				reviews: meta.reviews ?? []
			});
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			createFirkinErrors = {
				...createFirkinErrors,
				[key]: err instanceof Error ? err.message : 'Unknown error'
			};
		} finally {
			creatingFirkinFor = { ...creatingFirkinFor, [key]: false };
		}
	}

	type TvBuildPhase =
		| 'searching'
		| 'fetching_seasons'
		| 'fetching_episodes'
		| 'fetching_metadata'
		| 'waiting_pins'
		| 'creating_firkin'
		| 'completed'
		| 'error';

	interface TvBuildProgress {
		libraryId: string;
		jobKey: string;
		show: string;
		year?: number;
		phase: TvBuildPhase;
		message?: string;
		current?: number;
		total?: number;
		tmdbId?: string;
		tmdbTitle?: string;
		error?: string;
		completedFirkinId?: string;
		startedAt: string;
		updatedAt: string;
	}

	function showKey(libId: string, group: { show: string; year?: number }): string {
		return `${libId}::${group.show.toLowerCase()}::${group.year ?? ''}`;
	}

	function progressLabel(p: TvBuildProgress): string {
		if (p.message) return p.message;
		switch (p.phase) {
			case 'searching':
				return 'Searching TMDB…';
			case 'fetching_seasons':
				return 'Fetching seasons…';
			case 'fetching_episodes':
				return p.current !== undefined && p.total !== undefined
					? `Fetching episodes (${p.current}/${p.total})…`
					: 'Fetching episodes…';
			case 'fetching_metadata':
				return 'Fetching artists, trailers & reviews…';
			case 'waiting_pins':
				return p.current !== undefined && p.total !== undefined
					? `Waiting for IPFS pins (${p.current}/${p.total})…`
					: 'Waiting for IPFS pins…';
			case 'creating_firkin':
				return 'Creating firkin…';
			case 'completed':
				return 'Done — firkin built';
			case 'error':
				return p.error ?? 'Failed';
		}
	}

	/// Per-library polling. When a libraries page mounts (or a new library
	/// row appears in $libsStore) we start a 2-second loop that pulls
	/// `/api/libraries/:id/tv-builds`, mirroring the server's progress map
	/// into `tvBuildJobs[libraryId][jobKey]`. The loop self-stops when no
	/// non-terminal jobs remain — clicking a new "Match TMDB & build
	/// firkin" button restarts it via `ensureTvBuildPolling`.
	const tvBuildPollers = new Map<string, number>();
	let tvBuildJobs = $state<Record<string, Record<string, TvBuildProgress>>>({});
	/// Keys we've already auto-navigated for in this tab — keeps the
	/// "completed → goto /catalog/<id>" handler from firing again after
	/// the firkin page itself triggers another poll.
	const navigatedJobs = new Set<string>();
	/// Keys whose terminal "Done" badge has been acknowledged in this tab.
	/// Used to decide whether to auto-navigate (only when the user kicked
	/// off the build from this same tab) vs. to render a "View firkin"
	/// link (when the build was already in flight when the page mounted).
	const ownedJobs = new Set<string>();

	async function pollTvBuilds(libId: string): Promise<TvBuildProgress[]> {
		const res = await fetch(`/api/libraries/${encodeURIComponent(libId)}/tv-builds`, {
			cache: 'no-store'
		});
		if (!res.ok) {
			throw new Error(`HTTP ${res.status}`);
		}
		return (await res.json()) as TvBuildProgress[];
	}

	function applyTvBuildSnapshot(libId: string, snapshot: TvBuildProgress[]) {
		const next: Record<string, TvBuildProgress> = {};
		for (const p of snapshot) {
			next[p.jobKey] = p;
			if (
				p.phase === 'completed' &&
				p.completedFirkinId &&
				ownedJobs.has(p.jobKey) &&
				!navigatedJobs.has(p.jobKey)
			) {
				navigatedJobs.add(p.jobKey);
				const firkinId = p.completedFirkinId;
				void goto(`${base}/catalog/${encodeURIComponent(firkinId)}`);
			}
		}
		tvBuildJobs = { ...tvBuildJobs, [libId]: next };
	}

	function hasActiveBuild(libId: string): boolean {
		const map = tvBuildJobs[libId];
		if (!map) return false;
		return Object.values(map).some((p) => p.phase !== 'completed' && p.phase !== 'error');
	}

	function ensureTvBuildPolling(libId: string) {
		if (tvBuildPollers.has(libId)) return;
		const handle = window.setInterval(async () => {
			try {
				const snapshot = await pollTvBuilds(libId);
				applyTvBuildSnapshot(libId, snapshot);
				if (!hasActiveBuild(libId)) {
					const h = tvBuildPollers.get(libId);
					if (h !== undefined) {
						window.clearInterval(h);
						tvBuildPollers.delete(libId);
					}
				}
			} catch {
				// ignore — next tick retries
			}
		}, 2000);
		tvBuildPollers.set(libId, handle);
	}

	/// Hand the structured TV scan group off to the backend. The actual
	/// TMDB search, season/episode fetch, pin waiting, and firkin create
	/// all happen there in a `tokio::spawn`ed task — leaving the page or
	/// reloading does not interrupt the build, and the polling loop
	/// re-hydrates the in-progress badge from `/api/libraries/:id/tv-builds`.
	async function startTvFirkinBuild(
		libId: string,
		group: {
			show: string;
			year?: number;
			seasons: Array<{ season: number; entries: ScanEntry[] }>;
		}
	) {
		const key = showKey(libId, group);
		const files: Array<{ path: string; season: number; episode: number }> = [];
		for (const season of group.seasons) {
			for (const entry of season.entries) {
				const tv = entry.extractedTvQuery;
				if (!tv) continue;
				files.push({ path: entry.path, season: tv.season, episode: tv.episode });
			}
		}
		ownedJobs.add(key);
		try {
			const res = await fetch(`/api/libraries/${encodeURIComponent(libId)}/tv-build`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ show: group.show, year: group.year, files })
			});
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const initial = (await res.json()) as TvBuildProgress;
			tvBuildJobs = {
				...tvBuildJobs,
				[libId]: { ...(tvBuildJobs[libId] ?? {}), [initial.jobKey]: initial }
			};
			ensureTvBuildPolling(libId);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			tvBuildJobs = {
				...tvBuildJobs,
				[libId]: {
					...(tvBuildJobs[libId] ?? {}),
					[key]: {
						libraryId: libId,
						jobKey: key,
						show: group.show,
						year: group.year,
						phase: 'error',
						error: message,
						startedAt: new Date().toISOString(),
						updatedAt: new Date().toISOString()
					}
				}
			};
		}
	}

	/// One-shot DELETE that clears all completed/errored jobs for a
	/// library — used by the "Hide" button on terminal badges and on
	/// page mount to keep the table tidy after a successful build.
	async function clearTerminalTvBuilds(libId: string) {
		try {
			await fetch(`/api/libraries/${encodeURIComponent(libId)}/tv-builds`, {
				method: 'DELETE'
			});
		} catch {
			// best-effort
		}
		const next: Record<string, TvBuildProgress> = {};
		const current = tvBuildJobs[libId] ?? {};
		for (const [k, v] of Object.entries(current)) {
			if (v.phase !== 'completed' && v.phase !== 'error') next[k] = v;
		}
		tvBuildJobs = { ...tvBuildJobs, [libId]: next };
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB', 'TB'];
		let value = bytes / 1024;
		let i = 0;
		while (value >= 1024 && i < units.length - 1) {
			value /= 1024;
			i++;
		}
		return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[i]}`;
	}

	async function remove(id: string) {
		deletingId = id;
		try {
			await librariesService.remove(id);
		} catch (err) {
			librariesService.state.update((s) => ({
				...s,
				error: err instanceof Error ? err.message : 'Unknown error'
			}));
		} finally {
			deletingId = null;
		}
	}

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}

	type ScanGroup =
		| {
				kind: 'show';
				show: string;
				year?: number;
				seasons: Array<{ season: number; entries: ScanEntry[] }>;
		  }
		| { kind: 'flat'; label: string; entries: ScanEntry[] };

	function groupEntries(entries: ScanEntry[]): ScanGroup[] {
		const showGroups = new Map<
			string,
			{ show: string; year?: number; seasons: Map<number, ScanEntry[]> }
		>();
		const movieEntries: ScanEntry[] = [];
		const otherEntries: ScanEntry[] = [];

		for (const entry of entries) {
			if (entry.extractedTvQuery) {
				const key = entry.extractedTvQuery.show.toLowerCase();
				let group = showGroups.get(key);
				if (!group) {
					group = {
						show: entry.extractedTvQuery.show,
						year: entry.extractedTvQuery.year,
						seasons: new Map()
					};
					showGroups.set(key, group);
				}
				const season = entry.extractedTvQuery.season;
				let bucket = group.seasons.get(season);
				if (!bucket) {
					bucket = [];
					group.seasons.set(season, bucket);
				}
				bucket.push(entry);
			} else if (entry.extractedQuery) {
				movieEntries.push(entry);
			} else {
				otherEntries.push(entry);
			}
		}

		const result: ScanGroup[] = [];

		const sortedShows = [...showGroups.values()].sort((a, b) => a.show.localeCompare(b.show));
		for (const group of sortedShows) {
			const seasons = [...group.seasons.entries()]
				.sort(([a], [b]) => a - b)
				.map(([season, list]) => ({
					season,
					entries: [...list].sort(
						(a, b) => (a.extractedTvQuery?.episode ?? 0) - (b.extractedTvQuery?.episode ?? 0)
					)
				}));
			result.push({ kind: 'show', show: group.show, year: group.year, seasons });
		}

		if (movieEntries.length > 0) {
			movieEntries.sort((a, b) =>
				(a.extractedQuery?.title ?? '').localeCompare(b.extractedQuery?.title ?? '')
			);
			result.push({ kind: 'flat', label: 'Movies', entries: movieEntries });
		}
		if (otherEntries.length > 0) {
			otherEntries.sort((a, b) => a.relative_path.localeCompare(b.relative_path));
			result.push({ kind: 'flat', label: 'Other', entries: otherEntries });
		}

		return result;
	}
</script>

<Modal open={$modalStore.open} maxWidth="max-w-7xl" onclose={close}>
	<div class="flex flex-col gap-6">
		<header class="flex items-center justify-between gap-4">
			<div>
				<h2 class="text-2xl font-bold">Libraries</h2>
				<p class="text-sm text-base-content/60">
					Each library is a directory on this machine. Browse to an existing folder to use it as a
					library, or pick a parent and create a new subfolder.
				</p>
			</div>
			<button
				class="btn btn-outline btn-sm"
				onclick={() => librariesService.refresh()}
				disabled={$libsStore.loading}
			>
				Refresh
			</button>
		</header>

		{#if $libsStore.error}
			<div class="alert alert-error">
				<span>{$libsStore.error}</span>
			</div>
		{/if}

		<section class="card border border-base-content/10 bg-base-200 p-4">
			<h2 class="mb-3 text-lg font-semibold">Add a library</h2>
			<form class="flex flex-col gap-3" onsubmit={submit}>
				<div class="form-control">
					<span class="label-text mb-1 text-xs">Directory</span>
					<DirectoryPicker
						value={pickedDir}
						disabled={creating}
						onChange={(p) => (pickedDir = p)}
					/>
				</div>
				<label class="form-control">
					<span class="label-text text-xs">
						New subfolder (optional — created inside the picked directory)
					</span>
					<input
						type="text"
						class="input-bordered input input-sm"
						placeholder="leave empty to use the picked folder"
						bind:value={newSubfolder}
						disabled={creating}
					/>
				</label>
				<div class="form-control">
					<span class="label-text mb-1 text-xs">
						Kinds (which catalog types this library contains)
					</span>
					<div class="flex flex-wrap gap-2">
						{#each LIBRARY_ADDONS as kind (kind)}
							<label
								class={classNames('btn btn-sm', {
									'btn-primary': newAddons.includes(kind),
									'btn-outline': !newAddons.includes(kind)
								})}
							>
								<input
									type="checkbox"
									class="hidden"
									checked={newAddons.includes(kind)}
									disabled={creating}
									onchange={() => toggleNewAddon(kind)}
								/>
								{LIBRARY_ADDON_LABELS[kind]}
							</label>
						{/each}
					</div>
					<p class="mt-1 text-xs text-base-content/60">
						Files matching the selected kinds are pinned to IPFS on scan. Leave empty to skip
						pinning.
					</p>
				</div>
				<p class="text-xs text-base-content/60">
					Library directory: <span class="font-mono">{finalPath || '—'}</span>
				</p>
				<div>
					<button
						type="submit"
						class={classNames('btn btn-sm btn-primary', {
							'btn-disabled': creating || !finalPath
						})}
						disabled={creating || !finalPath}
					>
						{creating ? 'Creating…' : 'Create'}
					</button>
				</div>
			</form>
			{#if createError}
				<p class="mt-2 text-sm text-error">{createError}</p>
			{/if}
		</section>

		<section class="flex flex-col gap-3">
			<h2 class="text-lg font-semibold">Existing libraries</h2>
			{#if $libsStore.loading && $libsStore.libraries.length === 0}
				<p class="text-sm text-base-content/60">Loading…</p>
			{:else if $libsStore.libraries.length === 0}
				<p class="text-sm text-base-content/60">No libraries yet.</p>
			{:else}
				<div class="overflow-x-auto rounded-box border border-base-content/10">
					<table class="table table-sm">
						<thead>
							<tr>
								<th>Path</th>
								<th>Kinds</th>
								<th>Created</th>
								<th>Last scanned</th>
								<th class="w-24"></th>
							</tr>
						</thead>
						<tbody>
							{#each $libsStore.libraries as lib (lib.id)}
								<tr>
									<td class="font-mono text-xs break-all">{lib.path}</td>
									<td class="text-xs">
										{#if editingKindsFor === lib.id}
											<div class="flex flex-wrap items-center gap-1">
												{#each LIBRARY_ADDONS as kind (kind)}
													<label
														class={classNames('btn btn-xs', {
															'btn-primary': editAddons.includes(kind),
															'btn-outline': !editAddons.includes(kind)
														})}
													>
														<input
															type="checkbox"
															class="hidden"
															checked={editAddons.includes(kind)}
															disabled={savingKinds}
															onchange={() => toggleEditAddon(kind)}
														/>
														{LIBRARY_ADDON_LABELS[kind]}
													</label>
												{/each}
												<button
													class="btn btn-xs btn-primary"
													disabled={savingKinds}
													onclick={() => saveKinds(lib.id)}
												>
													{savingKinds ? 'Saving…' : 'Save'}
												</button>
												<button
													class="btn btn-ghost btn-xs"
													disabled={savingKinds}
													onclick={cancelEditKinds}
												>
													Cancel
												</button>
											</div>
											{#if addonsError}
												<p class="mt-1 text-error">{addonsError}</p>
											{/if}
										{:else if (lib.addons ?? []).length === 0}
											<button class="btn btn-ghost btn-xs" onclick={() => startEditKinds(lib)}>
												Set kinds
											</button>
										{:else}
											<button
												class="badge flex flex-wrap gap-1 badge-outline badge-sm"
												onclick={() => startEditKinds(lib)}
											>
												{(lib.addons ?? []).map((k) => LIBRARY_ADDON_LABELS[k] ?? k).join(', ')}
											</button>
										{/if}
									</td>
									<td class="text-xs text-base-content/60">{formatDate(lib.created_at)}</td>
									<td class="text-xs text-base-content/60">
										{lib.last_scanned_at ? formatDate(lib.last_scanned_at) : '—'}
									</td>
									<td class="text-right">
										<div class="flex justify-end gap-1">
											<button
												class="btn btn-ghost btn-xs"
												onclick={() => scan(lib.id)}
												disabled={scanningId === lib.id}
											>
												{scanningId === lib.id ? 'Scanning…' : 'Scan'}
											</button>
											<button
												class="btn text-error btn-ghost btn-xs"
												onclick={() => remove(lib.id)}
												disabled={deletingId === lib.id}
											>
												{deletingId === lib.id ? 'Removing…' : 'Remove'}
											</button>
										</div>
									</td>
								</tr>
								{#if scanErrors[lib.id]}
									<tr>
										<td colspan="5" class="bg-base-100">
											<div class="my-2 alert alert-error">
												<span class="text-sm">{scanErrors[lib.id]}</span>
												<button class="btn btn-ghost btn-xs" onclick={() => clearScan(lib.id)}>
													Dismiss
												</button>
											</div>
										</td>
									</tr>
								{:else if pinsErrors[lib.id]}
									<tr>
										<td colspan="5" class="bg-base-100">
											<div class="my-2 alert alert-warning">
												<span class="text-sm">Pins: {pinsErrors[lib.id]}</span>
											</div>
										</td>
									</tr>
								{:else if scanResults[lib.id]}
									{@const cidByPath = pinIndex(libPins[lib.id])}
									<tr>
										<td colspan="5" class="bg-base-100 p-3">
											<div class="flex flex-col gap-2">
												<div class="flex items-center justify-between gap-2">
													<p class="text-xs text-base-content/70">
														{scanResults[lib.id].total_files} files —
														{formatBytes(scanResults[lib.id].total_size)} total
													</p>
													<button class="btn btn-ghost btn-xs" onclick={() => clearScan(lib.id)}>
														Hide
													</button>
												</div>
												{#if scanResults[lib.id].entries.length === 0}
													<p class="text-xs text-base-content/60">No files in this directory.</p>
												{:else}
													<div
														class="max-h-96 overflow-y-auto rounded border border-base-content/10"
													>
														<table class="table table-xs">
															<thead class="sticky top-0 bg-base-200">
																<tr>
																	<th>File</th>
																	<th class="w-72">CID</th>
																	<th class="w-24">MIME</th>
																	<th class="w-20 text-right">Size</th>
																	<th class="w-40">Show / Title</th>
																	<th class="w-12">S</th>
																	<th class="w-12">E</th>
																	<th class="w-56">TMDB match</th>
																	<th class="w-32"></th>
																</tr>
															</thead>
															<tbody>
																{#snippet entryRow(entry: ScanEntry)}
																	{@const cid = cidByPath.get(entry.path)}
																	<tr>
																		<td class="font-mono text-xs break-all">
																			{entry.relative_path}
																		</td>
																		<td class="font-mono text-xs break-all">
																			{#if cid}
																				{cid}
																			{:else}
																				<span class="text-base-content/40">pinning…</span>
																			{/if}
																		</td>
																		<td class="font-mono text-xs">{entry.mime}</td>
																		<td class="text-right text-xs">
																			{formatBytes(entry.size)}
																		</td>
																		<td class="text-xs">
																			{#if entry.extractedTvQuery}
																				{entry.extractedTvQuery.show}{entry.extractedTvQuery.year
																					? ` (${entry.extractedTvQuery.year})`
																					: ''}
																			{:else if entry.extractedQuery}
																				{entry.extractedQuery.title}{entry.extractedQuery.year
																					? ` (${entry.extractedQuery.year})`
																					: ''}
																			{:else}
																				<span class="text-base-content/30">—</span>
																			{/if}
																		</td>
																		<td class="text-xs">
																			{#if entry.extractedTvQuery}
																				{String(entry.extractedTvQuery.season).padStart(2, '0')}
																			{:else}
																				<span class="text-base-content/30">—</span>
																			{/if}
																		</td>
																		<td class="text-xs">
																			{#if entry.extractedTvQuery}
																				{String(entry.extractedTvQuery.episode).padStart(2, '0')}
																			{:else}
																				<span class="text-base-content/30">—</span>
																			{/if}
																		</td>
																		<td class="text-xs">
																			{#if entry.tmdbMatch}
																				<a
																					class="link link-hover"
																					href={`https://www.themoviedb.org/movie/${entry.tmdbMatch.tmdbId}`}
																					target="_blank"
																					rel="noreferrer"
																					title={entry.tmdbMatch.overview ?? ''}
																				>
																					{entry.tmdbMatch.title}{entry.tmdbMatch.year
																						? ` (${entry.tmdbMatch.year})`
																						: ''}
																				</a>
																			{:else if entry.extractedTvQuery}
																				<span class="text-base-content/40">tmdb lookup pending</span
																				>
																			{:else if entry.extractedQuery}
																				<span class="text-base-content/40">no match</span>
																			{:else}
																				<span class="text-base-content/30">—</span>
																			{/if}
																		</td>
																		<td class="text-xs">
																			{#if entry.tmdbMatch}
																				<button
																					class="btn btn-xs btn-primary"
																					disabled={creatingFirkinFor[entry.path]}
																					onclick={() => createFirkinFromMatch(lib.id, entry, cid)}
																					title={cid
																						? undefined
																						: 'IPFS pin still in progress — clicking will wait for the pin to complete before creating the firkin'}
																				>
																					{creatingFirkinFor[entry.path]
																						? cid
																							? 'Creating…'
																							: 'Waiting for pin…'
																						: cid
																							? 'Create firkin'
																							: 'Create firkin (waits for pin)'}
																				</button>
																			{:else}
																				<span class="text-base-content/30">—</span>
																			{/if}
																		</td>
																	</tr>
																	{#if createFirkinErrors[entry.path]}
																		<tr>
																			<td colspan="9" class="bg-base-100">
																				<div class="my-1 alert py-1 alert-error">
																					<span class="text-xs">
																						Create firkin failed: {createFirkinErrors[entry.path]}
																					</span>
																				</div>
																			</td>
																		</tr>
																	{/if}
																{/snippet}
																{#each groupEntries(scanResults[lib.id].entries) as group, gi (gi)}
																	{#if group.kind === 'show'}
																		{@const origKey = showKey(lib.id, group)}
																		{@const effShow = editedShowNames[origKey] ?? group.show}
																		{@const effGroup = { ...group, show: effShow }}
																		{@const sKey = showKey(lib.id, effGroup)}
																		{@const job = tvBuildJobs[lib.id]?.[sKey]}
																		{@const inFlight =
																			!!job && job.phase !== 'completed' && job.phase !== 'error'}
																		{@const seasonCount = group.seasons.length}
																		{@const episodeCount = group.seasons.reduce(
																			(acc, s) => acc + s.entries.length,
																			0
																		)}
																		{@const preflight = preflightStates[origKey]}
																		<tr class="bg-base-200/60">
																			<td colspan="9" class="text-sm font-semibold">
																				<div class="flex items-center justify-between gap-2">
																					<div class="flex flex-col gap-1">
																						<div class="flex items-baseline gap-2">
																							<input
																								type="text"
																								class="input-bordered input input-xs w-72"
																								value={effShow}
																								oninput={(e) =>
																									onShowNameInput(
																										lib.id,
																										origKey,
																										(e.target as HTMLInputElement).value,
																										group.year
																									)}
																								title="Edit the term that will be searched against TMDB"
																							/>
																							{#if group.year}
																								<span
																									class="text-xs font-normal text-base-content/60"
																									>({group.year})</span
																								>
																							{/if}
																						</div>
																						<span class="text-xs font-normal text-base-content/60">
																							{seasonCount} season{seasonCount === 1 ? '' : 's'} ·
																							{episodeCount} episode{episodeCount === 1 ? '' : 's'} found
																						</span>
																						{#if preflight}
																							{#if preflight.status === 'searching'}
																								<span
																									class="text-xs font-normal text-base-content/40"
																								>
																									Checking TMDB…
																								</span>
																							{:else if preflight.status === 'typing'}
																								<span
																									class="text-xs font-normal text-base-content/40"
																								>
																									Editing — preflight will rerun
																								</span>
																							{:else if preflight.status === 'found'}
																								<a
																									class="link text-xs font-normal text-success link-hover"
																									href={`https://www.themoviedb.org/tv/${preflight.tmdbId}`}
																									target="_blank"
																									rel="noreferrer"
																								>
																									→ TMDB #{preflight.tmdbId}
																									{preflight.tmdbTitle}{preflight.tmdbYear
																										? ` (${preflight.tmdbYear})`
																										: ''}{preflight.seasonCount !== undefined
																										? ` · ${preflight.seasonCount} season${preflight.seasonCount === 1 ? '' : 's'}`
																										: ''}
																								</a>
																							{:else if preflight.status === 'no_match'}
																								<span class="text-xs font-normal text-error">
																									No TMDB match — build will error
																								</span>
																							{:else if preflight.status === 'error'}
																								<span class="text-xs font-normal text-error">
																									Preflight: {preflight.error}
																								</span>
																							{/if}
																						{/if}
																					</div>
																					<div class="flex items-center gap-2">
																						{#if job?.phase === 'completed' && job.completedFirkinId}
																							<a
																								class="btn btn-xs btn-success"
																								href={`${base}/catalog/${encodeURIComponent(job.completedFirkinId)}`}
																							>
																								View firkin
																							</a>
																						{/if}
																						<button
																							class="btn btn-xs btn-primary"
																							disabled={inFlight}
																							onclick={() => startTvFirkinBuild(lib.id, effGroup)}
																						>
																							{inFlight
																								? progressLabel(job!)
																								: job?.phase === 'completed'
																									? 'Re-run match'
																									: 'Match TMDB & build firkin'}
																						</button>
																					</div>
																				</div>
																				{#if inFlight && job?.total !== undefined && job.total > 0}
																					{@const pct = Math.round(
																						((job.current ?? 0) / job.total) * 100
																					)}
																					<progress
																						class="progress mt-1 w-full progress-primary"
																						value={pct}
																						max="100"
																					></progress>
																				{/if}
																				{#if job?.phase === 'error'}
																					<div class="mt-1 flex items-center gap-2">
																						<p class="text-xs font-normal text-error">
																							{job.error ?? 'Failed'}
																						</p>
																						<button
																							class="btn btn-ghost btn-xs"
																							onclick={() => clearTerminalTvBuilds(lib.id)}
																						>
																							Dismiss
																						</button>
																					</div>
																				{/if}
																			</td>
																		</tr>
																	{:else}
																		<tr class="bg-base-200/60">
																			<td colspan="9" class="text-sm font-semibold"
																				>{group.label}</td
																			>
																		</tr>
																		{#each group.entries as entry (entry.path)}
																			{@render entryRow(entry)}
																		{/each}
																	{/if}
																{/each}
															</tbody>
														</table>
													</div>
												{/if}
											</div>
										</td>
									</tr>
								{/if}
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</section>
	</div>
</Modal>
