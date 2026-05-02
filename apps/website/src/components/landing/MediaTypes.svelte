<script lang="ts">
	import { Icon, type IconName } from 'cloud-ui';

	type Kind = {
		icon: IconName;
		label: string;
		source: string;
		comingSoon?: boolean;
	};

	const kinds: Kind[] = [
		{ icon: 'delapouite/popcorn', label: 'Movies', source: 'TMDB · OMDb' },
		{ icon: 'delapouite/film-strip', label: 'TV shows', source: 'TMDB' },
		{ icon: 'delapouite/musical-notes', label: 'Music', source: 'MusicBrainz · LRCLIB' },
		{ icon: 'delapouite/tv', label: 'YouTube', source: 'yt-dlp · InnerTube' },
		{ icon: 'delapouite/bookshelf', label: 'Books', source: 'Local libraries', comingSoon: true },
		{ icon: 'delapouite/gamepad', label: 'Games', source: 'Local libraries', comingSoon: true }
	];
</script>

<section class="border-b border-base-300 bg-base-100">
	<div class="mx-auto w-full max-w-6xl px-4 py-16">
		<div class="mb-8 max-w-2xl">
			<h2 class="text-3xl font-bold tracking-tight md:text-4xl">
				Everything you watch and listen to
			</h2>
			<p class="mt-3 text-base-content/70">
				One catalog, one detail page, one player — for every type of media in your library.
			</p>
		</div>

		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
			{#each kinds as kind (kind.label)}
				<div
					class={[
						'relative flex flex-col items-center rounded-box border border-base-300 bg-base-100 p-5 text-center',
						kind.comingSoon && 'border-dashed opacity-60'
					]}
					aria-disabled={kind.comingSoon ? 'true' : undefined}
				>
					{#if kind.comingSoon}
						<span
							class="absolute top-2 right-2 badge badge-xs text-[0.65rem] font-semibold tracking-wide uppercase badge-warning"
						>
							Soon
						</span>
					{/if}
					<div
						class="mb-2 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 text-primary"
					>
						<Icon name={kind.icon} size={24} />
					</div>
					<div class="font-semibold">{kind.label}</div>
					<div class="mt-1 text-xs text-base-content/60">
						{kind.comingSoon ? 'Coming soon' : kind.source}
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>
