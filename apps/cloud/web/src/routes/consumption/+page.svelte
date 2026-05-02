<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { listMediaTrackers, type MediaTracker } from '$lib/media-tracker.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import type { Firkin } from '$lib/firkins.service';

	const userIdentityState = userIdentityService.state;

	interface FirkinGroup {
		firkinId: string;
		firkin: Firkin | undefined;
		albumRow: MediaTracker | null;
		trackRows: MediaTracker[];
		totalSeconds: number;
		lastPlayedAt: string;
	}

	let trackers = $state<MediaTracker[]>([]);
	let firkinsById = $state<Record<string, Firkin>>({});
	let loading = $state(false);
	let error = $state<string | null>(null);
	let lastLoadedAddress: string | null = null;
	let expandedFirkin = $state<string | null>(null);

	// Group raw tracker rows by firkin: the row with no trackId (movies, TV,
	// non-music streams, or the legacy album-level pre-per-track row) becomes
	// the "album" row for the firkin; rows with trackId become per-track
	// children. Total time per firkin sums both, so albums whose listening
	// happened before per-track tracking landed still show the right total.
	let groups = $derived.by<FirkinGroup[]>(() => {
		const map = new Map<string, FirkinGroup>();
		for (const row of trackers) {
			let group = map.get(row.firkinId);
			if (!group) {
				group = {
					firkinId: row.firkinId,
					firkin: firkinsById[row.firkinId],
					albumRow: null,
					trackRows: [],
					totalSeconds: 0,
					lastPlayedAt: row.last_played_at
				};
				map.set(row.firkinId, group);
			}
			if (row.trackId) {
				group.trackRows.push(row);
			} else {
				group.albumRow = row;
			}
			group.totalSeconds += row.totalSeconds;
			if (row.last_played_at > group.lastPlayedAt) {
				group.lastPlayedAt = row.last_played_at;
			}
		}
		const list = Array.from(map.values());
		for (const g of list) {
			g.trackRows.sort((a, b) => b.totalSeconds - a.totalSeconds);
		}
		list.sort((a, b) => (a.lastPlayedAt < b.lastPlayedAt ? 1 : -1));
		return list;
	});

	$effect(() => {
		const address = $userIdentityState.identity?.address;
		if (!address) {
			trackers = [];
			firkinsById = {};
			lastLoadedAddress = null;
			return;
		}
		if (lastLoadedAddress === address) return;
		lastLoadedAddress = address;
		void load(address);
	});

	async function load(address: string) {
		loading = true;
		error = null;
		try {
			const [tList, fRes] = await Promise.all([
				listMediaTrackers(address),
				fetch('/api/firkins', { cache: 'no-store' })
			]);
			const firkins = fRes.ok ? ((await fRes.json()) as Firkin[]) : [];
			const map: Record<string, Firkin> = {};
			for (const f of firkins) map[f.id] = f;
			trackers = tList;
			firkinsById = map;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function truncateCid(cid: string): string {
		if (cid.length <= 18) return cid;
		return `${cid.slice(0, 10)}…${cid.slice(-6)}`;
	}

	function copyHash(cid: string) {
		void navigator.clipboard?.writeText(cid).catch(() => {
			// silent — clipboard may be unavailable
		});
	}

	function formatDuration(totalSeconds: number): string {
		const s = Math.max(0, Math.floor(totalSeconds));
		const h = Math.floor(s / 3600);
		const m = Math.floor((s % 3600) / 60);
		const sec = s % 60;
		if (h > 0) return `${h}h ${m}m`;
		if (m > 0) return `${m}m ${sec}s`;
		return `${sec}s`;
	}

	function formatRelative(iso: string): string {
		const then = new Date(iso).getTime();
		if (!Number.isFinite(then)) return '—';
		const diff = Date.now() - then;
		const sec = Math.round(diff / 1000);
		if (sec < 60) return `${Math.max(0, sec)}s ago`;
		const min = Math.round(sec / 60);
		if (min < 60) return `${min}m ago`;
		const hr = Math.round(min / 60);
		if (hr < 24) return `${hr}h ago`;
		const day = Math.round(hr / 24);
		return `${day}d ago`;
	}

	function formatAbsolute(iso: string): string {
		const d = new Date(iso);
		return Number.isFinite(d.getTime()) ? d.toLocaleString() : iso;
	}

	function totalAcrossRows(rows: MediaTracker[]): number {
		let sum = 0;
		for (const r of rows) sum += r.totalSeconds;
		return sum;
	}

	onMount(() => {
		const address = $userIdentityState.identity?.address;
		if (address) {
			lastLoadedAddress = address;
			void load(address);
		}
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Consumption</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-semibold">Consumption</h1>
		<p class="text-sm text-base-content/60">
			Playback time per firkin, accumulated by the right-side player. The tracker writes a heartbeat
			every 10 seconds while a firkin file is streaming and not paused; pauses, buffering, and
			connection teardown stop time accruing. Music firkins expand to show per-track listening time.
		</p>
	</header>

	{#if !$userIdentityState.identity}
		<div class="alert alert-warning">
			<span>Sign in on the Profile page to see your consumption.</span>
		</div>
	{:else}
		{#if error}
			<div class="alert alert-error"><span>{error}</span></div>
		{/if}

		<section class="grid grid-cols-1 gap-4 sm:grid-cols-3">
			<div class="card border border-base-content/10 bg-base-200">
				<div class="card-body p-4">
					<div class="text-xs text-base-content/60 uppercase">Tracked firkins</div>
					<div class="text-2xl font-semibold">{groups.length}</div>
				</div>
			</div>
			<div class="card border border-base-content/10 bg-base-200">
				<div class="card-body p-4">
					<div class="text-xs text-base-content/60 uppercase">Total time</div>
					<div class="text-2xl font-semibold">{formatDuration(totalAcrossRows(trackers))}</div>
				</div>
			</div>
			<div class="card border border-base-content/10 bg-base-200">
				<div class="card-body p-4">
					<div class="text-xs text-base-content/60 uppercase">Last played</div>
					<div class="text-2xl font-semibold">
						{groups[0] ? formatRelative(groups[0].lastPlayedAt) : '—'}
					</div>
				</div>
			</div>
		</section>

		<section class="card border border-base-content/10 bg-base-200">
			<div class="card-body p-0">
				<div class="overflow-x-auto">
					<table class="table table-zebra">
						<thead>
							<tr>
								<th class="w-16">Poster</th>
								<th>Title</th>
								<th class="w-40">IPFS hash</th>
								<th class="w-32 text-right">Total time</th>
								<th class="w-40 text-right">Last played</th>
							</tr>
						</thead>
						<tbody>
							{#if loading && trackers.length === 0}
								<tr>
									<td colspan="5" class="text-center text-base-content/60">Loading…</td>
								</tr>
							{:else if trackers.length === 0}
								<tr>
									<td colspan="5" class="text-center text-base-content/60">
										No consumption recorded yet — start playing a firkin from the right-side player
										to see it here.
									</td>
								</tr>
							{:else}
								{#each groups as group (group.firkinId)}
									{@const firkin = group.firkin}
									{@const poster = firkin?.images?.[0]?.url}
									{@const title = firkin?.title ?? '(unknown firkin)'}
									{@const detailHref = firkin
										? `${base}/catalog/${encodeURIComponent(group.firkinId)}`
										: null}
									{@const expanded = expandedFirkin === group.firkinId}
									{@const hasTracks = group.trackRows.length > 0}
									<tr>
										<td>
											{#if poster}
												<img
													src={poster}
													alt={title}
													class="h-16 w-12 shrink-0 rounded object-cover"
													loading="lazy"
												/>
											{:else}
												<div
													class="flex h-16 w-12 shrink-0 items-center justify-center rounded bg-base-300 text-base-content/30"
												>
													<svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
														<path
															d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"
														/>
													</svg>
												</div>
											{/if}
										</td>
										<td>
											<div class="flex flex-col gap-0.5">
												<div class="flex items-center gap-2">
													{#if hasTracks}
														<button
															type="button"
															class="btn px-1 btn-ghost btn-xs"
															aria-label={expanded ? 'Collapse tracks' : 'Expand tracks'}
															title={expanded ? 'Collapse tracks' : 'Expand tracks'}
															onclick={() => (expandedFirkin = expanded ? null : group.firkinId)}
														>
															{expanded ? '▾' : '▸'}
														</button>
													{/if}
													{#if detailHref}
														<a
															href={detailHref}
															class="link font-medium link-hover"
															title="Open detail page"
														>
															{title}
														</a>
													{:else}
														<span class="font-medium text-base-content/60">{title}</span>
													{/if}
												</div>
												<div class="flex flex-wrap items-center gap-1 text-xs text-base-content/60">
													{#if firkin}
														<span class="badge badge-ghost badge-sm">{firkin.addon}</span>
														{#if firkin.year}
															<span>{firkin.year}</span>
														{/if}
													{/if}
													{#if hasTracks}
														<span class="badge badge-sm badge-info">
															{group.trackRows.length} track{group.trackRows.length === 1
																? ''
																: 's'}
														</span>
													{/if}
												</div>
											</div>
										</td>
										<td>
											<button
												type="button"
												class="font-mono text-xs text-base-content/70 hover:text-base-content"
												title={`${group.firkinId} (click to copy)`}
												onclick={() => copyHash(group.firkinId)}
											>
												{truncateCid(group.firkinId)}
											</button>
										</td>
										<td class="text-right font-mono text-sm">
											{formatDuration(group.totalSeconds)}
										</td>
										<td
											class="text-right text-sm text-base-content/70"
											title={formatAbsolute(group.lastPlayedAt)}
										>
											{formatRelative(group.lastPlayedAt)}
										</td>
									</tr>
									{#if expanded && hasTracks}
										{#each group.trackRows as track (track.id)}
											<tr class="bg-base-100/40">
												<td></td>
												<td colspan="2" class="pl-12">
													<div class="flex flex-col gap-0.5">
														<span class="text-sm">
															{track.trackTitle ?? '(untitled track)'}
														</span>
														{#if track.trackId}
															<span class="font-mono text-[10px] text-base-content/40">
																{track.trackId}
															</span>
														{/if}
													</div>
												</td>
												<td class="text-right font-mono text-sm">
													{formatDuration(track.totalSeconds)}
												</td>
												<td
													class="text-right text-sm text-base-content/70"
													title={formatAbsolute(track.last_played_at)}
												>
													{formatRelative(track.last_played_at)}
												</td>
											</tr>
										{/each}
									{/if}
								{/each}
							{/if}
						</tbody>
					</table>
				</div>
			</div>
		</section>
	{/if}
</div>
