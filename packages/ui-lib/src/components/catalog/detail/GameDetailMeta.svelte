<script lang="ts">
	import type { CatalogGame } from 'ui-lib/types/catalog.type';

	interface Props {
		item: CatalogGame;
	}

	let { item }: Props = $props();

	let consoleName = $derived(item.metadata.consoleName);
	let developer = $derived(item.metadata.developer);
	let publisher = $derived(item.metadata.publisher);
	let genre = $derived(item.metadata.genre);
	let released = $derived(item.metadata.released);
	let numAchievements = $derived(item.metadata.numAchievements);
	let points = $derived(item.metadata.points);
	let achievements = $derived(item.metadata.achievements);
	let imageBoxArtUrl = $derived(item.metadata.imageBoxArtUrl);
	let imageTitleUrl = $derived(item.metadata.imageTitleUrl);
	let imageIngameUrl = $derived(item.metadata.imageIngameUrl);
</script>

<div class="flex flex-col gap-3">
	<div class="grid grid-cols-2 gap-2 text-sm">
		<div>
			<span class="opacity-50">Console:</span>
			<span class="font-medium">{consoleName}</span>
		</div>
		{#if developer}
			<div>
				<span class="opacity-50">Developer:</span>
				<span class="font-medium">{developer}</span>
			</div>
		{/if}
		{#if publisher}
			<div>
				<span class="opacity-50">Publisher:</span>
				<span class="font-medium">{publisher}</span>
			</div>
		{/if}
		{#if genre}
			<div>
				<span class="opacity-50">Genre:</span>
				<span class="font-medium">{genre}</span>
			</div>
		{/if}
		{#if released}
			<div>
				<span class="opacity-50">Released:</span>
				<span class="font-medium">{released}</span>
			</div>
		{/if}
		<div>
			<span class="opacity-50">Achievements:</span>
			<span class="font-medium">{numAchievements} ({points} pts)</span>
		</div>
	</div>

	{#if imageBoxArtUrl || imageTitleUrl || imageIngameUrl}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Screenshots</h3>
			<div class="grid grid-cols-3 gap-1">
				{#if imageBoxArtUrl}
					<img src={imageBoxArtUrl} alt="Box art" class="w-full rounded object-cover" loading="lazy" />
				{/if}
				{#if imageTitleUrl}
					<img src={imageTitleUrl} alt="Title screen" class="w-full rounded object-cover" loading="lazy" />
				{/if}
				{#if imageIngameUrl}
					<img src={imageIngameUrl} alt="In-game" class="w-full rounded object-cover" loading="lazy" />
				{/if}
			</div>
		</div>
	{/if}

	{#if achievements.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">
				Achievements ({achievements.length})
			</h3>
			<div class="flex flex-col gap-1">
				{#each achievements.slice(0, 20) as achievement}
					<div class="flex items-center gap-2 rounded p-1.5 text-sm hover:bg-base-200">
						<img src={achievement.badgeUrl} alt={achievement.title} class="h-8 w-8 rounded" loading="lazy" />
						<div class="flex-1">
							<p class="text-xs font-medium">{achievement.title}</p>
							<p class="text-xs opacity-50">{achievement.description}</p>
						</div>
						<span class="badge badge-accent badge-xs">{achievement.points}</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
