<script lang="ts">
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type { RaGameMetadata } from 'addons/retroachievements/types';

	interface Props {
		game: RaGameMetadata;
		details: RaGameMetadata | null;
		detailsLoading: boolean;
		onback: () => void;
	}

	let { game, details, detailsLoading, onback }: Props = $props();

	let heroImage = $derived(
		details?.imageTitleUrl || details?.imageIngameUrl || details?.imageBoxArtUrl
	);
</script>

<DetailPageLayout>
		<button class="btn self-start btn-ghost btn-sm" onclick={onback}>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
			</svg>
			Back
		</button>

		<h1 class="text-xl font-bold">{game.title}</h1>

		{#if heroImage}
			<img src={heroImage} alt={game.title} class="w-full rounded-lg object-cover" />
		{:else if game.imageIconUrl}
			<img
				src={game.imageIconUrl}
				alt={game.title}
				class="aspect-square w-full max-w-sm rounded-lg object-cover"
			/>
		{:else}
			<div
				class="flex aspect-square w-full max-w-sm items-center justify-center rounded-lg bg-base-200"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-16 w-16 text-base-content/20"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875s-2.25.84-2.25 1.875c0 .369.128.713.349 1.003.215.283.401.604.401.959v0a.64.64 0 01-.657.643 48.39 48.39 0 01-4.163-.3c.186 1.613.293 3.25.315 4.907a.656.656 0 01-.658.663v0c-.355 0-.676-.186-.959-.401a1.647 1.647 0 00-1.003-.349c-1.036 0-1.875 1.007-1.875 2.25s.84 2.25 1.875 2.25c.369 0 .713-.128 1.003-.349.283-.215.604-.401.959-.401v0c.31 0 .555.26.532.57a48.039 48.039 0 01-.642 5.056c1.518.19 3.058.309 4.616.354a.64.64 0 00.657-.643v0c0-.355-.186-.676-.401-.959a1.647 1.647 0 01-.349-1.003c0-1.035 1.008-1.875 2.25-1.875 1.243 0 2.25.84 2.25 1.875 0 .369-.128.713-.349 1.003-.215.283-.4.604-.4.959v0c0 .333.277.599.61.58a48.1 48.1 0 005.427-.63 48.05 48.05 0 00.582-4.717.532.532 0 00-.533-.57v0c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.035 0-1.875-1.007-1.875-2.25s.84-2.25 1.875-2.25c.37 0 .713.128 1.003.349.283.215.604.401.959.401v0a.656.656 0 00.658-.663 48.422 48.422 0 00-.37-5.36c-1.886.342-3.81.574-5.766.689a.578.578 0 01-.61-.58v0z"
					/>
				</svg>
			</div>
		{/if}

		<p class="text-sm opacity-60">{game.consoleName}</p>

		{#if detailsLoading}
			<div class="flex items-center justify-center py-4">
				<span class="loading loading-sm loading-spinner"></span>
			</div>
		{:else if details}
			<div class="flex flex-col gap-1.5">
				{#if details.developer}
					<div class="flex items-center gap-1 text-sm">
						<span class="opacity-40">Developer:</span><span>{details.developer}</span>
					</div>
				{/if}
				{#if details.publisher}
					<div class="flex items-center gap-1 text-sm">
						<span class="opacity-40">Publisher:</span><span>{details.publisher}</span>
					</div>
				{/if}
				{#if details.genre}
					<div class="flex items-center gap-1 text-sm">
						<span class="opacity-40">Genre:</span><span>{details.genre}</span>
					</div>
				{/if}
				{#if details.released}
					<div class="flex items-center gap-1 text-sm">
						<span class="opacity-40">Released:</span><span>{details.released}</span>
					</div>
				{/if}
				{#if details.numDistinctPlayers}
					<div class="flex items-center gap-1 text-sm">
						<span class="opacity-40">Players:</span><span
							>{details.numDistinctPlayers.toLocaleString()}</span
						>
					</div>
				{/if}

				<div class="flex flex-wrap gap-1 pt-1">
					{#if details.numAchievements > 0}
						<span class="badge badge-sm badge-info">{details.numAchievements} achievements</span>
					{/if}
					{#if details.points > 0}
						<span class="badge badge-ghost badge-sm">{details.points} points</span>
					{/if}
				</div>

				{#if details.imageBoxArtUrl}
					<img
						src={details.imageBoxArtUrl}
						alt="Box art"
						class="mt-2 w-full rounded-lg"
						loading="lazy"
					/>
				{/if}
				{#if details.imageIngameUrl}
					<img
						src={details.imageIngameUrl}
						alt="In-game screenshot"
						class="w-full rounded-lg"
						loading="lazy"
					/>
				{/if}
				{#if details.imageTitleUrl}
					<img
						src={details.imageTitleUrl}
						alt="Title screen"
						class="w-full rounded-lg"
						loading="lazy"
					/>
				{/if}
			</div>

			{#if details.achievements && details.achievements.length > 0}
				<div class="flex flex-col gap-0.5 pt-2">
					<div class="flex items-center justify-between">
						<h4 class="text-sm font-semibold opacity-50">Achievements</h4>
						<span class="text-xs opacity-30">{details.achievements.length} total</span>
					</div>
					{#each details.achievements as achievement (achievement.id)}
						<div class="flex items-center gap-2 rounded px-1 py-1 hover:bg-base-200">
							{#if achievement.badgeUrl}
								<img
									src={achievement.badgeUrl}
									alt={achievement.title}
									class="h-8 w-8 rounded"
									loading="lazy"
								/>
							{/if}
							<div class="min-w-0 flex-1">
								<p class="truncate text-sm font-medium">{achievement.title}</p>
								<p class="truncate text-xs opacity-40">{achievement.description}</p>
							</div>
							<span class="text-xs opacity-30">{achievement.points}pts</span>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
</DetailPageLayout>
