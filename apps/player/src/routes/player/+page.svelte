<script lang="ts">
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
	import IpfsConfigPanel from '$components/IpfsConfigPanel.svelte';
	import FirkinIpfsPlayer from '$components/FirkinIpfsPlayer.svelte';
	import { ipfsConfigStore } from '$ipfs/config.svelte';
	import { getPlayerIpfsClient, catText, type PlayerIpfsClient } from '$ipfs/client';

	type Status =
		| { kind: 'idle' }
		| { kind: 'connecting' }
		| { kind: 'fetching'; cid: string }
		| { kind: 'loaded'; firkin: Firkin }
		| { kind: 'error'; message: string };

	let cidInput = $state('');
	let configOpen = $state(false);
	let status = $state<Status>({ kind: 'idle' });
	let client: PlayerIpfsClient | null = $state(null);
	let peerCount = $state(0);
	let peerTimer: ReturnType<typeof setInterval> | null = null;

	const firkin = $derived<Firkin | null>(status.kind === 'loaded' ? status.firkin : null);
	const firkinKind = $derived(firkin ? addonKind(firkin.addon) : null);
	const configured = $derived(ipfsConfigStore.configured);

	$effect(() => {
		if (!client) return;
		peerTimer = setInterval(() => {
			peerCount = client?.peerCount() ?? 0;
		}, 1500);
		return () => {
			if (peerTimer) clearInterval(peerTimer);
			peerTimer = null;
		};
	});

	async function ensureClient(): Promise<PlayerIpfsClient> {
		if (client) return client;
		status = { kind: 'connecting' };
		const c = await getPlayerIpfsClient({
			bootstrapMultiaddrs: ipfsConfigStore.bootstrapMultiaddrs,
			swarmKey: ipfsConfigStore.swarmKey
		});
		client = c;
		return c;
	}

	async function loadCid() {
		const cid = cidInput.trim();
		if (!cid) return;
		if (!configured) {
			configOpen = true;
			return;
		}

		try {
			const c = await ensureClient();
			status = { kind: 'fetching', cid };
			const text = await catText(c, cid);
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
			<button type="button" class="btn btn-ghost btn-sm" onclick={() => (configOpen = true)}>
				IPFS settings
			</button>
		</div>
		<p class="mt-2 text-xs text-base-content/60">
			{#if !configured}
				No swarm key / bootstrap configured — click <strong>IPFS settings</strong> to paste yours.
			{:else if client}
				Connected to {peerCount} peer{peerCount === 1 ? '' : 's'} on the private swarm.
			{:else}
				Configured. Connection opens on first load.
			{/if}
		</p>
	</section>

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
	{:else if status.kind !== 'connecting' && status.kind !== 'fetching'}
		<div class="card border border-base-content/10 bg-base-200 p-6 text-center">
			<p class="text-sm text-base-content/70">
				Paste a firkin CID above and click Load to fetch it directly from the private IPFS swarm.
				The player never talks to the cloud HTTP API — the firkin body and every attached file are
				fetched as UnixFS blocks via libp2p.
			</p>
		</div>
	{/if}
</div>

<IpfsConfigPanel open={configOpen} onClose={() => (configOpen = false)} />
