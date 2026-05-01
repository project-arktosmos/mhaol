<script lang="ts">
	import { onMount } from 'svelte';
	import {
		CatalogPageHeader,
		CatalogDescriptionCard,
		CatalogIdentityCard,
		CatalogVersionHistoryCard,
		CatalogFilesTable,
		CatalogImagesCard,
		CatalogTrailersDisplay,
		FirkinArtistsSection,
		addonKind,
		type Firkin
	} from 'cloud-ui';
	import FirkinIpfsPlayer from '$components/FirkinIpfsPlayer.svelte';
	import { playerIpfsConfig, playerIpfsConfigured, playerIpfsDiagnostic } from '$ipfs/config';
	import { getPlayerIpfsClient, catText, type PlayerIpfsClient } from '$ipfs/client';

	type Status =
		| { kind: 'idle' }
		| { kind: 'connecting' }
		| { kind: 'fetching'; cid: string }
		| { kind: 'loaded'; firkin: Firkin }
		| { kind: 'error'; message: string };

	let cidInput = $state('');
	let status = $state<Status>({ kind: 'idle' });
	let client = $state<PlayerIpfsClient | null>(null);
	let connectError = $state<string | null>(null);
	let peerCount = $state(0);

	const firkin = $derived<Firkin | null>(status.kind === 'loaded' ? status.firkin : null);
	const firkinKind = $derived(firkin ? addonKind(firkin.addon) : null);

	onMount(() => {
		if (!playerIpfsConfigured) return;
		void connect();
		const t = setInterval(() => {
			peerCount = client?.peerCount() ?? 0;
		}, 1500);
		return () => clearInterval(t);
	});

	async function connect() {
		try {
			const c = await getPlayerIpfsClient(playerIpfsConfig);
			client = c;
			peerCount = c.peerCount();
		} catch (err) {
			connectError = err instanceof Error ? err.message : String(err);
		}
	}

	async function loadCid() {
		const cid = cidInput.trim();
		if (!cid) return;
		if (!playerIpfsConfigured) return;
		try {
			if (!client) {
				status = { kind: 'connecting' };
				await connect();
			}
			if (!client) throw new Error(connectError ?? 'IPFS client unavailable');
			status = { kind: 'fetching', cid };
			const text = await catText(client, cid);
			let parsed: unknown;
			try {
				parsed = JSON.parse(text);
			} catch {
				throw new Error('Fetched bytes are not valid JSON — is this a firkin CID?');
			}
			if (!parsed || typeof parsed !== 'object') {
				throw new Error('Firkin body must be a JSON object');
			}
			const f = parsed as Firkin;
			if (!f.id) f.id = cid;
			if (!Array.isArray(f.files)) f.files = [];
			if (!Array.isArray(f.images)) f.images = [];
			if (!Array.isArray(f.artists)) f.artists = [];
			status = { kind: 'loaded', firkin: f };
		} catch (err) {
			status = {
				kind: 'error',
				message: err instanceof Error ? err.message : String(err)
			};
		}
	}

	function reset() {
		status = { kind: 'idle' };
		cidInput = '';
	}
</script>

<svelte:head>
	<title>Mhaol Player</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	{#if !playerIpfsConfigured}
		<div class="alert alert-error">
			<div class="flex flex-col gap-1">
				<span class="font-semibold">IPFS connection not configured</span>
				<span class="text-xs">
					The player reads the rendezvous bootstrap multiaddrs and the swarm key from disk at
					startup (via <code>scripts/run-vite.mjs</code>). Right now:
				</span>
				<ul class="ml-4 list-disc text-xs">
					<li>
						Bootstrap addrs found:
						<strong>{playerIpfsDiagnostic.bootstrapMultiaddrs}</strong>
						{#if playerIpfsDiagnostic.bootstrapMultiaddrs === 0}
							— start <code>pnpm app:rendezvous</code> first so it writes
							<code>$DATA_DIR/rendezvous/bootstrap.multiaddr</code>, or set
							<code>RENDEZVOUS_BOOTSTRAP</code>.
						{/if}
					</li>
					<li>
						Swarm key: <strong>{playerIpfsDiagnostic.swarmKey}</strong>
						{#if playerIpfsDiagnostic.swarmKey !== 'present'}
							— expected at <code>$DATA_DIR/swarm.key</code> (or set
							<code>IPFS_SWARM_KEY_FILE</code>).
						{/if}
					</li>
				</ul>
				<span class="text-xs">
					Restart <code>pnpm dev:player</code> after the rendezvous is up.
				</span>
			</div>
		</div>
	{:else}
		<section class="card border border-base-content/10 bg-base-200 p-4">
			<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
				Open a firkin from IPFS
			</h2>
			<div class="flex flex-wrap items-center gap-2">
				<input
					type="text"
					class="input-bordered input input-sm min-w-[280px] flex-1 font-mono text-xs"
					placeholder="bafy... (firkin CID)"
					bind:value={cidInput}
					onkeydown={(e) => {
						if (e.key === 'Enter') void loadCid();
					}}
				/>
				<button
					type="button"
					class="btn btn-sm btn-primary"
					disabled={status.kind === 'connecting' || status.kind === 'fetching'}
					onclick={loadCid}
				>
					{#if status.kind === 'connecting'}
						Connecting…
					{:else if status.kind === 'fetching'}
						Fetching…
					{:else}
						Load
					{/if}
				</button>
			</div>
			<p class="mt-2 text-xs text-base-content/60">
				{#if connectError}
					Connection error: <span class="text-error">{connectError}</span>
				{:else if client}
					Connected to {peerCount} peer{peerCount === 1 ? '' : 's'} on the private swarm (rendezvous:
					{playerIpfsConfig.bootstrapMultiaddrs.length} bootstrap addr{playerIpfsConfig
						.bootstrapMultiaddrs.length === 1
						? ''
						: 's'}).
				{:else}
					Connecting to rendezvous…
				{/if}
			</p>
		</section>
	{/if}

	{#if status.kind === 'error'}
		<div class="alert alert-error">
			<span>{status.message}</span>
			<button type="button" class="btn btn-ghost btn-xs" onclick={reset}>Reset</button>
		</div>
	{/if}

	{#if firkin}
		<CatalogPageHeader
			title={firkin.title}
			addon={firkin.addon}
			kindLabel={firkinKind}
			year={firkin.year}
		/>

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
			<aside class="flex flex-col gap-4">
				{#each firkin.images as image, i (image.url || i)}
					<img
						src={image.url}
						alt={firkin.title}
						loading="lazy"
						class="w-full rounded-md object-cover"
					/>
				{/each}

				<FirkinArtistsSection
					artists={firkin.artists}
					emptyLabel="No people or groups attached."
					singleColumn
				/>
			</aside>

			<section class="flex flex-col gap-6">
				<CatalogDescriptionCard description={firkin.description} />

				{#if client}
					<FirkinIpfsPlayer {firkin} {client} />
				{/if}

				<CatalogIdentityCard
					cid={firkin.id}
					createdAt={firkin.created_at}
					updatedAt={firkin.updated_at}
					version={firkin.version ?? 0}
				/>

				<CatalogVersionHistoryCard versionHashes={firkin.version_hashes ?? []} />

				{#if firkin.trailers && firkin.trailers.length > 0}
					<CatalogTrailersDisplay trailers={firkin.trailers} />
				{/if}

				<CatalogImagesCard images={firkin.images} />

				<CatalogFilesTable files={firkin.files} />
			</section>
		</div>
	{:else if playerIpfsConfigured && status.kind !== 'connecting' && status.kind !== 'fetching'}
		<div class="card border border-base-content/10 bg-base-200 p-6 text-center">
			<p class="text-sm text-base-content/70">
				Paste a firkin CID above and click Load to fetch it directly from the private IPFS swarm.
				The player never talks to the cloud HTTP API — the firkin body and every attached file are
				fetched as UnixFS blocks via libp2p.
			</p>
		</div>
	{/if}
</div>
