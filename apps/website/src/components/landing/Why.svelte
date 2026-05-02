<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Card = {
		icon: IconName;
		eyebrow: string;
		title: string;
		body: string;
		points: string[];
	};

	const cards: Card[] = [
		{
			icon: 'lorc/crossed-chains',
			eyebrow: 'Permanence',
			title: 'Your library doesn’t expire',
			body: 'A series leaving the catalog. An album yanked from a streaming app. A movie locked behind another price hike. None of that touches a Mhaol library — your files live on a drive you own, and Mhaol just makes them feel like the apps you used to pay for.',
			points: [
				'Movies, TV, and music in one viewer — no app per service',
				'Plays even when the internet is down; the library is local',
				'Every record is content-addressed, so it can never be silently swapped'
			]
		},
		{
			icon: 'delapouite/keyring',
			eyebrow: 'Privacy',
			title: 'No accounts. No cloud. No middleman.',
			body: 'There is no signup. There is no SaaS. A cryptographic identity you generate in your own browser signs you into your own cloud — export it, import it, regenerate it whenever you like. Catalog lookups, subtitles, lyrics, and torrent searches all run through your hardware, not a third party that logs them.',
			points: [
				'Self-owned identity (secp256k1, EIP-191) — no email, no password',
				'TMDB / OMDb / MusicBrainz / YouTube proxied through your machine',
				'Subtitles and lyrics fetched on demand, then pinned locally'
			]
		},
		{
			icon: 'delapouite/wifi-router',
			eyebrow: 'Continuity',
			title: 'Every screen, one library',
			body: 'Pick any device — laptop, home server, Android TV, phone. They mesh over your LAN automatically and present the same catalog, the same identity, and the same library. No file shares to mount, no syncing tools, no “where did I leave off” because every device is a peer.',
			points: [
				'mDNS finds nodes on your LAN automatically',
				'Headless server for the closet; tray app for the laptop; TV + mobile for the couch',
				'Bookmarks and play state ride with the firkin, not the device'
			]
		}
	];
</script>

<section id="why" class="border-b border-base-300 bg-base-100">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-3xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">Streaming, on your terms</h2>
			<p class="mt-3 text-base-content/70">
				Mhaol gives you back what subscription services and walled-garden apps slowly took away —
				ownership, privacy, and a library that follows you across every screen you own.
			</p>
		</div>

		<div class="grid gap-5 md:grid-cols-3">
			{#each cards as card (card.title)}
				<div
					class="flex flex-col rounded-box border border-base-300 bg-base-100 p-6 transition hover:-translate-y-0.5 hover:shadow-md"
				>
					<div
						class="mb-4 flex h-12 w-12 items-center justify-center rounded-md bg-primary/10 text-primary"
					>
						<Icon name={card.icon} size={26} />
					</div>
					<div class="text-xs font-semibold tracking-wide text-primary uppercase">
						{card.eyebrow}
					</div>
					<h3 class="mt-1 text-xl font-semibold">{card.title}</h3>
					<p class="mt-2 text-sm text-base-content/70">{card.body}</p>
					<ul class="mt-4 space-y-1.5 border-t border-base-300 pt-4 text-sm text-base-content/80">
						{#each card.points as point (point)}
							<li class="flex items-start gap-2">
								<span class="mt-1 inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-primary"></span>
								<span>{point}</span>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	</div>
</section>
