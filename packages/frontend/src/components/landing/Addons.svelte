<script lang="ts">
	const addons = [
		{
			title: 'TMDB',
			description:
				'Movie and TV metadata from The Movie Database — posters, backdrops, cast, ratings, and recommendations.',
			icon: 'M3.375 19.5h17.25a1.125 1.125 0 0 0 0-2.25H3.375a1.125 1.125 0 0 0 0 2.25ZM5.625 15V5.625a1.875 1.875 0 0 1 1.875-1.875h9a1.875 1.875 0 0 1 1.875 1.875V15m-13.5 0h13.5'
		},
		{
			title: 'MusicBrainz',
			description:
				'Comprehensive music metadata — artists, albums, tracks, release dates, and cover art from the open music encyclopedia.',
			icon: 'M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 0 1-1.632 2.163l-1.32.377a1.803 1.803 0 1 1-.99-3.467l2.31-.66a2.25 2.25 0 0 0 1.632-2.163Zm0 0V4.103A2.25 2.25 0 0 0 17.868 2.1l-6.987 2A2.25 2.25 0 0 0 9.25 6.292V16.5m0 0v3.75a2.25 2.25 0 0 1-1.632 2.163l-1.32.377a1.803 1.803 0 0 1-.99-3.467l2.31-.66A2.25 2.25 0 0 0 9.25 16.84V16.5Z'
		},
		{
			title: 'YouTube',
			description:
				'YouTube video extraction and downloading — search, preview, and download videos in any format and quality.',
			icon: 'M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653z'
		},
		{
			title: 'LRCLIB',
			description:
				'Synchronized lyrics — fetch time-stamped lyrics for your music library and display them in the player.',
			icon: 'M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z'
		},
		{
			title: 'Torrent Search',
			description:
				'Torrent search across public indexes — find, compare, and add torrents with size, seeder, and trust indicators.',
			icon: 'M21 21l-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z'
		}
	];
</script>

<section id="addons" class="bg-base-200 py-20 lg:py-28">
	<div class="mx-auto max-w-6xl px-6">
		<div class="mb-12 text-center">
			<h2 class="text-3xl font-bold sm:text-4xl">Powered by Addons</h2>
			<p class="mt-4 text-base-content/60">
				Modular integrations that bring rich metadata, lyrics, search, and more to your media
				server.
			</p>
		</div>
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
			{#each addons as addon}
				<div class="card bg-base-100 shadow-md transition-shadow hover:shadow-lg">
					<div class="card-body">
						<div class="flex items-center gap-3">
							<div
								class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-secondary/10"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="1.5"
									stroke="currentColor"
									class="h-5 w-5 text-secondary"
								>
									<path stroke-linecap="round" stroke-linejoin="round" d={addon.icon} />
								</svg>
							</div>
							<h3 class="card-title text-lg">{addon.title}</h3>
						</div>
						<p class="mt-2 text-sm text-base-content/60">
							{addon.description}
						</p>
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>
