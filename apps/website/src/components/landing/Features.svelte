<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Feature = {
		icon: IconName;
		title: string;
		body: string;
	};

	const features: Feature[] = [
		{
			icon: 'delapouite/cube',
			title: 'Content-addressed firkins',
			body: 'Every record (movie, show, album, book, game) gets a stable CID derived from its serialized body. Edits roll the version forward without losing history.'
		},
		{
			icon: 'delapouite/share',
			title: 'Embedded IPFS node',
			body: 'libp2p over TCP and WebSockets, Bitswap, Kademlia DHT, and a pre-shared private network — your swarm, your peers, no public gateway needed.'
		},
		{
			icon: 'delapouite/server-rack',
			title: 'Self-hosted, all-in-one',
			body: 'A single binary embeds the SurrealDB store, Axum API, IPFS node, torrent client, yt-dlp, and the SPA. Run it on your laptop or a tiny server.'
		},
		{
			icon: 'delapouite/network-bars',
			title: 'LAN-aware discovery',
			body: 'mDNS pulls in other Mhaol nodes on the same network automatically, so your phone, TV, and desktop see the same library without configuration.'
		},
		{
			icon: 'delapouite/files',
			title: 'Smart library scanning',
			body: 'Walks library directories, fingerprints files, and groups them into firkins per type — TV seasons, music albums, books, games.'
		},
		{
			icon: 'lorc/aerial-signal',
			title: 'Subs + lyrics, on demand',
			body: 'OpenSubtitles converts SRT to VTT and pins to IPFS the moment you start a stream. LRCLIB synced lyrics are persisted with your albums.'
		},
		{
			icon: 'delapouite/film-strip',
			title: 'Catalog browsing built-in',
			body: 'TMDB, OMDb, MusicBrainz, YouTube, and torrent search — all proxied through your cloud so API keys stay server-side.'
		},
		{
			icon: 'delapouite/keyring',
			title: 'Identity you own',
			body: 'Browser-resident secp256k1 identity signs an EIP-191 message to register against your cloud. Export, import, or regenerate at any time.'
		}
	];
</script>

<section id="features" class="border-b border-base-300 bg-base-100">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-2xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">A library that travels with you</h2>
			<p class="mt-3 text-base-content/70">
				Mhaol does the boring infrastructure so your media just works — across devices, across
				networks, and across time.
			</p>
		</div>

		<div class="grid gap-5 md:grid-cols-2 lg:grid-cols-3">
			{#each features as feature (feature.title)}
				<div
					class="rounded-box border border-base-300 bg-base-100 p-5 transition hover:-translate-y-0.5 hover:shadow-md"
				>
					<div
						class="mb-3 flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary"
					>
						<Icon name={feature.icon} size={22} />
					</div>
					<h3 class="text-lg font-semibold">{feature.title}</h3>
					<p class="mt-1 text-sm text-base-content/70">{feature.body}</p>
				</div>
			{/each}
		</div>
	</div>
</section>
