<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Upcoming = {
		icon: IconName;
		title: string;
		body: string;
		points: string[];
	};

	const upcoming: Upcoming[] = [
		{
			icon: 'delapouite/bookshelf',
			title: 'Books',
			body: 'A reading library that lives next to the rest of your media — EPUBs pinned to your cloud, scanned out of a folder you point Mhaol at, and synced to whatever device is in your hand.',
			points: [
				'EPUB / PDF library scanning',
				'Reading progress that follows the firkin, not the device',
				'Cover art and metadata via OpenLibrary'
			]
		},
		{
			icon: 'delapouite/gamepad',
			title: 'Games',
			body: 'A ROM and game library with the same content-addressed semantics as the rest of Mhaol — point at a folder, get a clean catalog with cover art, then play retro games right inside Mhaol with bundled web emulators or hand off to your desktop emulator of choice.',
			points: [
				'ROM library scanning by platform',
				'Bundled in-browser emulators for NES, SNES, Mega Drive, GB / GBA, and more',
				'Cover art + metadata for every system',
				'Optional launch hand-off to your installed desktop emulator'
			]
		},
		{
			icon: 'delapouite/compact-disc',
			title: 'Optical discs',
			body: 'CDs, DVDs, and Blu-rays as first-class citizens of your library — play them in place, rip them into firkins so the discs can go on the shelf, or burn a curated playlist back out for the car or a friend.',
			points: [
				'Play CD / DVD / Blu-ray straight from the drive',
				'Rip to a firkin with cover art and tracklist',
				'Burn audio CDs and data discs from your library'
			]
		},
		{
			icon: 'delapouite/three-friends',
			title: 'Watch and listen together',
			body: 'Open a room, drop in a mixed playlist of movies, episodes, albums, and YouTube clips, and everyone connected plays in lockstep. The host drives the timeline, friends co-edit the queue, and chat lives in the same space — tied to the moment on screen.',
			points: [
				'Mixed playlists across movies, TV, music, and YouTube',
				'Synchronized playback — host’s seek bar drives every device',
				'Collaborative queue editing and reactions',
				'Built-in chat anchored to the playback timeline'
			]
		},
		{
			icon: 'lorc/tune-pitch',
			title: 'Audio EQ and video tuning',
			body: 'Sculpt the sound and the picture to your room and your screen — a per-device audio equalizer for music and films, plus video presets (gamma, contrast, denoise, sharpening) you can save and switch between.',
			points: [
				'Per-device audio equalizer with savable profiles',
				'Video presets for screen, room, and time of day',
				'Loudness normalization across the library'
			]
		},
		{
			icon: 'delapouite/network-bars',
			title: 'ed2k client and search',
			body: 'An embedded eDonkey / eMule client sitting next to the existing torrent stack — search, queue, and pull files from the long tail of the network without leaving the Mhaol catalog.',
			points: [
				'In-app ed2k search across Kad and server lists',
				'Downloads pinned to your IPFS node on completion',
				'Same firkin auto-attach the torrent client already uses'
			]
		},
		{
			icon: 'lorc/rss',
			title: 'Torrent RSS for new episodes',
			body: 'Subscribe a TV show to a torrent RSS feed and Mhaol grabs new episodes the moment they drop — picked, streamability-checked, downloaded, and pinned to the right season firkin without you opening the app.',
			points: [
				'Per-show RSS feed configuration',
				'Quality, language, and release-group filters',
				'Auto-attach to the matching firkin season'
			]
		},
		{
			icon: 'lorc/satellite',
			title: 'YouTube channel subscriptions',
			body: 'Follow the creators you actually watch via the public channel Atom feed. New uploads appear in your YouTube library, with an option to auto-download them to play offline next time the device is online.',
			points: [
				'Channel feed subscription — no Google login required',
				'Optional auto-download for offline playback',
				'Per-channel notification and retention policy'
			]
		},
		{
			icon: 'lorc/world',
			title: 'Full multilingual support',
			body: 'The catalog, the player, the Settings page, every modal — all translatable, with the right language picked up from your device locale or set explicitly on your identity. Catalog metadata, subtitles, and lyrics all follow the same preference.',
			points: [
				'Catalog metadata in your preferred language (TMDB / OMDb)',
				'Subtitles and lyrics filtered by language',
				'Translatable UI with community-contributed locales'
			]
		},
		{
			icon: 'lorc/checked-shield',
			title: 'Parental controls',
			body: 'Lock the catalog to age-appropriate content per profile, with TMDB and OMDb certifications driving the filter and a PIN that survives device restarts and identity exports. Kids see what they are allowed to see; parents see everything.',
			points: [
				'Per-profile age limit using TMDB / OMDb certifications',
				'PIN-protected toggle that survives identity export and re-import',
				'Filtered firkins disappear from search, recommendations, and rooms',
				'One-shot override codes for the “okay, just this once” case'
			]
		}
	];
</script>

<section id="coming-up" class="border-b border-base-300 bg-base-200">
	<div class="mx-auto w-full max-w-6xl px-4 py-20">
		<div class="mb-10 max-w-3xl">
			<div
				class="mb-3 inline-flex items-center gap-1.5 text-xs font-semibold tracking-wide text-primary uppercase"
			>
				<Icon name="delapouite/info" size={14} />
				Roadmap
			</div>
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">Coming up</h2>
			<p class="mt-3 text-base-content/70">
				Features that aren't shipping today but are on the roadmap. They show up in the catalog with
				a “Soon” marker until they're ready.
			</p>
		</div>

		<div class="grid gap-5 md:grid-cols-2 lg:grid-cols-3">
			{#each upcoming as item (item.title)}
				<div
					class="relative flex flex-col rounded-box border border-dashed border-base-300 bg-base-100/60 p-6"
					aria-disabled="true"
				>
					<span
						class="absolute top-4 right-4 badge gap-1 text-[0.65rem] font-semibold tracking-wide uppercase badge-warning"
					>
						<Icon name="delapouite/info" size={12} />
						Coming soon
					</span>

					<div class="flex items-center gap-4">
						<div
							class="flex h-12 w-12 items-center justify-center rounded-md bg-primary/10 text-primary opacity-60"
						>
							<Icon name={item.icon} size={26} />
						</div>
						<h3 class="text-xl font-semibold text-base-content/80">{item.title}</h3>
					</div>

					<p class="mt-4 text-sm text-base-content/70">{item.body}</p>

					<ul
						class="mt-4 space-y-1.5 border-t border-dashed border-base-300 pt-4 text-sm text-base-content/70"
					>
						{#each item.points as point (point)}
							<li class="flex items-start gap-2">
								<span class="mt-1 inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-base-content/30"
								></span>
								<span>{point}</span>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	</div>
</section>
