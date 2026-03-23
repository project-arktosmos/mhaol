<script lang="ts">
	import { _ } from 'svelte-i18n';

	const addons = [
		{
			key: 'tmdb',
			icon: 'M3.375 19.5h17.25a1.125 1.125 0 0 0 0-2.25H3.375a1.125 1.125 0 0 0 0 2.25ZM5.625 15V5.625a1.875 1.875 0 0 1 1.875-1.875h9a1.875 1.875 0 0 1 1.875 1.875V15m-13.5 0h13.5'
		},
		{
			key: 'musicbrainz',
			icon: 'M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 0 1-1.632 2.163l-1.32.377a1.803 1.803 0 1 1-.99-3.467l2.31-.66a2.25 2.25 0 0 0 1.632-2.163Zm0 0V4.103A2.25 2.25 0 0 0 17.868 2.1l-6.987 2A2.25 2.25 0 0 0 9.25 6.292V16.5m0 0v3.75a2.25 2.25 0 0 1-1.632 2.163l-1.32.377a1.803 1.803 0 0 1-.99-3.467l2.31-.66A2.25 2.25 0 0 0 9.25 16.84V16.5Z'
		},
		{
			key: 'retroachievements',
			icon: 'M16.5 18.75h-9m9 0a3 3 0 0 1 3 3h-15a3 3 0 0 1 3-3m9 0v-3.375c0-.621-.503-1.125-1.125-1.125h-.871M7.5 18.75v-3.375c0-.621.504-1.125 1.125-1.125h.872m5.007 0H9.497m5.007 0a7.454 7.454 0 0 1-.982-3.172M9.497 14.25a7.454 7.454 0 0 0 .981-3.172M5.25 4.236c-.982.143-1.954.317-2.916.52A6.003 6.003 0 0 0 7.73 9.728M5.25 4.236V4.5c0 2.108.966 3.99 2.48 5.228M5.25 4.236V2.721C7.456 2.41 9.71 2.25 12 2.25c2.291 0 4.545.16 6.75.47v1.516M18.75 4.236c.982.143 1.954.317 2.916.52A6.003 6.003 0 0 1 16.27 9.728M18.75 4.236V4.5c0 2.108-.966 3.99-2.48 5.228m0 0a6.04 6.04 0 0 1-2.019 1.11c-.753.253-1.548.39-2.25.39s-1.498-.138-2.25-.39a6.04 6.04 0 0 1-2.02-1.11'
		},
		{
			key: 'youtube',
			icon: 'M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653z'
		},
		{
			key: 'lrclib',
			icon: 'M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z'
		},
		{
			key: 'torrentSearch',
			icon: 'M21 21l-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z'
		}
	];
</script>

<section id="addons" class="bg-base-200 py-20 lg:py-28">
	<div class="mx-auto max-w-6xl px-6">
		<div class="mb-12 text-center">
			<h2 class="text-3xl font-bold sm:text-4xl">{$_('addons.heading')}</h2>
			<p class="mt-4 text-base-content/60">{$_('addons.subheading')}</p>
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
							<h3 class="card-title text-lg">{$_(`addons.${addon.key}.title`)}</h3>
						</div>
						<p class="mt-2 text-sm text-base-content/60">
							{$_(`addons.${addon.key}.description`)}
						</p>
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>
