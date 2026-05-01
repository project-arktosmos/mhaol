<script lang="ts">
	import FeatureCard from './FeatureCard.svelte';

	const features = [
		{
			title: 'Movies',
			description:
				'Browse your movie library with rich TMDB metadata, posters, and AI-powered torrent search to find and stream any film.',
			icon: 'M3.375 19.5h17.25a1.125 1.125 0 0 0 0-2.25H3.375a1.125 1.125 0 0 0 0 2.25ZM5.625 15V5.625a1.875 1.875 0 0 1 1.875-1.875h9a1.875 1.875 0 0 1 1.875 1.875V15m-13.5 0h13.5'
		},
		{
			title: 'TV Shows',
			description:
				'Track series, manage episodes and seasons, and stream TV shows with automatic metadata and torrent integration.',
			icon: 'M6 20.25h12m-7.5-3v3m3-3v3m-10.125-3h17.25c.621 0 1.125-.504 1.125-1.125V4.875c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125Z'
		},
		{
			title: 'Music',
			description:
				'Manage your music collection with MusicBrainz metadata, album art, artist info, and synchronized lyrics.',
			icon: 'M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 0 1-1.632 2.163l-1.32.377a1.803 1.803 0 1 1-.99-3.467l2.31-.66a2.25 2.25 0 0 0 1.632-2.163Zm0 0V4.103A2.25 2.25 0 0 0 17.868 2.1l-6.987 2A2.25 2.25 0 0 0 9.25 6.292V16.5m0 0v3.75a2.25 2.25 0 0 1-1.632 2.163l-1.32.377a1.803 1.803 0 0 1-.99-3.467l2.31-.66A2.25 2.25 0 0 0 9.25 16.84V16.5Z'
		},
		{
			title: 'Photos',
			description:
				'Organize and browse your photo collection with tagging, albums, and a beautiful gallery view.',
			icon: 'M2.25 15.75l5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z'
		},
		{
			title: 'YouTube',
			description:
				'Download and organize YouTube videos locally. Search, preview, and queue downloads with full format control.',
			icon: 'M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653z'
		},
		{
			title: 'Books',
			description:
				'Read EPUBs and PDFs, track your reading progress, and organize your book collection offline.',
			icon: 'M12 6.042A8.967 8.967 0 0 0 6 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 0 1 6 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 0 1 6-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0 0 18 18a8.967 8.967 0 0 0-6 2.292m0-14.25v14.25'
		}
	];
</script>

<section id="features" class="bg-base-200 py-20 lg:py-28">
	<div class="mx-auto max-w-6xl px-6">
		<div class="mb-12 text-center">
			<h2 class="text-3xl font-bold sm:text-4xl">Everything in One Place</h2>
			<p class="mt-4 text-base-content/60">
				Browse, organize, and stream all your media from a single self-hosted server.
			</p>
		</div>
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each features as feature}
				<FeatureCard title={feature.title} description={feature.description} icon={feature.icon} />
			{/each}
		</div>
	</div>
</section>
