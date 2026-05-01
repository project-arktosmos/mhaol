<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { listMediaTrackers, type MediaTracker } from '$lib/media-tracker.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import type { Firkin } from '$lib/firkins.service';

	const userIdentityState = userIdentityService.state;

	let trackers = $state<MediaTracker[]>([]);
	let firkinsById = $state<Record<string, Firkin>>({});
	let loading = $state(false);
	let error = $state<string | null>(null);
	let lastLoadedAddress: string | null = null;

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
			connection teardown stop time accruing.
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
					<div class="text-2xl font-semibold">{trackers.length}</div>
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
						{trackers[0] ? formatRelative(trackers[0].last_played_at) : '—'}
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
								{#each trackers as row (row.id)}
									{@const firkin = firkinsById[row.firkinId]}
									{@const poster = firkin?.images?.[0]?.url}
									{@const title = firkin?.title ?? '(unknown firkin)'}
									{@const detailHref = firkin
										? `${base}/catalog/${encodeURIComponent(row.firkinId)}`
										: null}
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
												<div class="flex flex-wrap items-center gap-1 text-xs text-base-content/60">
													{#if firkin}
														<span class="badge badge-ghost badge-sm">{firkin.addon}</span>
														{#if firkin.year}
															<span>{firkin.year}</span>
														{/if}
													{/if}
												</div>
											</div>
										</td>
										<td>
											<button
												type="button"
												class="font-mono text-xs text-base-content/70 hover:text-base-content"
												title={`${row.firkinId} (click to copy)`}
												onclick={() => copyHash(row.firkinId)}
											>
												{truncateCid(row.firkinId)}
											</button>
										</td>
										<td class="text-right font-mono text-sm">
											{formatDuration(row.totalSeconds)}
										</td>
										<td
											class="text-right text-sm text-base-content/70"
											title={formatAbsolute(row.last_played_at)}
										>
											{formatRelative(row.last_played_at)}
										</td>
									</tr>
								{/each}
							{/if}
						</tbody>
					</table>
				</div>
			</div>
		</section>
	{/if}
</div>
