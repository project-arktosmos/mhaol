<script lang="ts">
	import classNames from 'classnames';
	import type { RaGameMetadata } from 'addons/retroachievements/types';

	interface Props {
		game: RaGameMetadata;
		selected?: boolean;
		favorited?: boolean;
		pinned?: boolean;
		onselect?: (game: RaGameMetadata) => void;
	}

	let { game, selected = false, favorited = false, pinned = false, onselect }: Props = $props();

	let imgError = $state(false);

	let cardImageUrl = $derived(game.imageBoxArtUrl);
</script>

<div
	class={classNames('card-compact card bg-base-200 shadow-sm', {
		'ring-2 ring-primary': selected,
		'cursor-pointer transition-shadow hover:shadow-md': !!onselect
	})}
	onclick={() => onselect?.(game)}
	role={onselect ? 'button' : undefined}
	tabindex={onselect ? 0 : undefined}
	onkeydown={onselect
		? (e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					onselect?.(game);
				}
			}
		: undefined}
>
	<figure class="relative aspect-square overflow-hidden bg-base-300">
		{#if cardImageUrl && !imgError}
			<img
				src={cardImageUrl}
				alt={game.title}
				class="h-full w-full object-cover"
				loading="lazy"
				onerror={() => (imgError = true)}
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center text-base-content/20">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-12 w-12"
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
		{#if game.numAchievements > 0}
			<div class="absolute top-1 right-1">
				<span class="badge badge-xs badge-info">{game.numAchievements} ach.</span>
			</div>
		{/if}
		{#if favorited || pinned}
			<div class="absolute bottom-1.5 left-1.5 z-10 flex gap-1">
				{#if favorited}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4 text-red-500 drop-shadow"
						viewBox="0 0 24 24"
						fill="currentColor"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
						/>
					</svg>
				{/if}
				{#if pinned}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4 text-blue-400 drop-shadow"
						viewBox="0 0 24 24"
						fill="currentColor"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							fill-rule="evenodd"
							d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
							clip-rule="evenodd"
						/>
					</svg>
				{/if}
			</div>
		{/if}
	</figure>
	<div class="card-body gap-0.5">
		<h3 class="card-title truncate text-sm" title={game.title}>{game.title}</h3>
		<p class="truncate text-xs opacity-60" title={game.consoleName}>{game.consoleName}</p>
		<div class="flex items-center gap-1">
			{#if game.points > 0}
				<span class="text-xs opacity-40">{game.points} pts</span>
			{/if}
			{#if game.genre}
				<span class="badge badge-ghost badge-xs">{game.genre}</span>
			{/if}
		</div>
	</div>
</div>
