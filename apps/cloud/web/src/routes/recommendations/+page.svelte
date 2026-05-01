<script lang="ts">
	import { onMount } from 'svelte';
	import { listRecommendations, type Recommendation } from '$lib/recommendations.service';
	import { userIdentityService } from '$lib/user-identity.service';

	const userIdentityState = userIdentityService.state;

	let rows = $state<Recommendation[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let lastLoadedAddress: string | null = null;

	$effect(() => {
		const address = $userIdentityState.identity?.address;
		if (!address) {
			rows = [];
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
			rows = await listRecommendations(address);
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

	onMount(() => {
		const address = $userIdentityState.identity?.address;
		if (address) {
			lastLoadedAddress = address;
			void load(address);
		}
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Recommendations</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-semibold">Recommendations</h1>
		<p class="text-sm text-base-content/60">
			Items the catalog API has recommended to you, indexed by their virtual IPFS hash. Counts only
			update when you visit a real <code>/catalog/[ipfsHash]</code> detail page; virtual catalog pages
			don't contribute. Each source firkin contributes at most once per item. Rows disappear automatically
			once you bookmark or assign a torrent for the recommended item.
		</p>
	</header>

	{#if !$userIdentityState.identity}
		<div class="alert alert-warning">
			<span>Sign in on the Profile page to see your recommendations.</span>
		</div>
	{:else}
		{#if error}
			<div class="alert alert-error"><span>{error}</span></div>
		{/if}

		<section class="card border border-base-content/10 bg-base-200">
			<div class="card-body p-0">
				<div class="overflow-x-auto">
					<table class="table table-zebra">
						<thead>
							<tr>
								<th class="w-16">Poster</th>
								<th>Title</th>
								<th class="w-40">IPFS hash</th>
								<th class="w-20 text-right">Count</th>
							</tr>
						</thead>
						<tbody>
							{#if loading && rows.length === 0}
								<tr>
									<td colspan="4" class="text-center text-base-content/60">Loading…</td>
								</tr>
							{:else if rows.length === 0}
								<tr>
									<td colspan="4" class="text-center text-base-content/60">
										No recommendations yet — visit a movie, TV show, or album detail page to start
										collecting.
									</td>
								</tr>
							{:else}
								{#each rows as row (row.firkinId)}
									<tr>
										<td>
											{#if row.posterUrl}
												<img
													src={row.posterUrl}
													alt={row.title}
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
												<span class="font-medium">{row.title}</span>
												<div class="flex flex-wrap items-center gap-1 text-xs text-base-content/60">
													<span class="badge badge-ghost badge-sm">{row.addon}</span>
													{#if row.year}
														<span>{row.year}</span>
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
										<td class="text-right font-mono text-sm">{row.count}</td>
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
