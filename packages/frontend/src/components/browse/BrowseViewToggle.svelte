<script lang="ts">
	import { getContext } from 'svelte';
	import classNames from 'classnames';

	type BrowseViewMode = 'poster' | 'backdrop' | 'table';

	const browseViewMode = getContext<
		{ readonly value: BrowseViewMode; set: (mode: BrowseViewMode) => void } | undefined
	>('browseViewMode');

	let mode = $derived(browseViewMode?.value ?? 'poster');
</script>

{#if browseViewMode}
	<div class="flex gap-1">
		<button
			class={classNames('btn btn-outline btn-xs', {
				'btn-active': mode === 'poster'
			})}
			onclick={() => browseViewMode.set('poster')}
			aria-label="Poster view"
			title="Poster view"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M3.75 6A2.25 2.25 0 016 3.75h2.25A2.25 2.25 0 0110.5 6v2.25a2.25 2.25 0 01-2.25 2.25H6a2.25 2.25 0 01-2.25-2.25V6zM3.75 15.75A2.25 2.25 0 016 13.5h2.25a2.25 2.25 0 012.25 2.25V18a2.25 2.25 0 01-2.25 2.25H6A2.25 2.25 0 013.75 18v-2.25zM13.5 6a2.25 2.25 0 012.25-2.25H18A2.25 2.25 0 0120.25 6v2.25A2.25 2.25 0 0118 10.5h-2.25a2.25 2.25 0 01-2.25-2.25V6zM13.5 15.75a2.25 2.25 0 012.25-2.25H18a2.25 2.25 0 012.25 2.25V18A2.25 2.25 0 0118 20.25h-2.25A2.25 2.25 0 0113.5 18v-2.25z"
				/>
			</svg>
			Poster
		</button>
		<button
			class={classNames('btn btn-outline btn-xs', {
				'btn-active': mode === 'backdrop'
			})}
			onclick={() => browseViewMode.set('backdrop')}
			aria-label="Backdrop view"
			title="Backdrop view"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z"
				/>
			</svg>
			Backdrop
		</button>
		<button
			class={classNames('btn btn-outline btn-xs', {
				'btn-active': mode === 'table'
			})}
			onclick={() => browseViewMode.set('table')}
			aria-label="Table view"
			title="Table view"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M3.75 5.25h16.5m-16.5 4.5h16.5m-16.5 4.5h16.5m-16.5 4.5h16.5"
				/>
			</svg>
			Table
		</button>
	</div>
{/if}
