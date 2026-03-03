<script lang="ts">
	import classNames from 'classnames';
	import { libraryService } from '$services/library.service';
	import type { DirectoryEntry } from '$types/library.type';

	interface Props {
		onselect: (path: string, name: string) => void;
	}

	let { onselect }: Props = $props();

	const state = libraryService.state;

	function handleNavigate(path: string) {
		libraryService.browseDirectory(path);
	}

	function handleNavigateUp() {
		if ($state.browseParent) {
			libraryService.browseDirectory($state.browseParent);
		}
	}

	function handleSelect(entry: DirectoryEntry) {
		onselect(entry.path, entry.name);
	}

	function handleSelectCurrent() {
		const parts = $state.currentBrowsePath.split('/');
		const name = parts[parts.length - 1] || 'Root';
		onselect($state.currentBrowsePath, name);
	}
</script>

<div class="flex flex-col gap-3">
	<!-- Current path and navigation -->
	<div class="flex items-center gap-2">
		<button
			class={classNames('btn btn-ghost btn-sm', {
				'btn-disabled': !$state.browseParent
			})}
			disabled={!$state.browseParent}
			onclick={handleNavigateUp}
		>
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
			Up
		</button>
		<button class="btn btn-sm btn-primary" onclick={handleSelectCurrent}>
			Select This Directory
		</button>
	</div>

	<!-- Current path display -->
	<div class="rounded-lg bg-base-300 px-3 py-2 font-mono text-sm">
		{$state.currentBrowsePath || '...'}
	</div>

	<!-- Error -->
	{#if $state.browseError}
		<div class="alert-sm alert alert-error">
			<span>{$state.browseError}</span>
		</div>
	{/if}

	<!-- Directory listing -->
	<div class="max-h-64 overflow-y-auto rounded-lg bg-base-100">
		{#if $state.browsing}
			<div class="flex justify-center p-4">
				<span class="loading loading-sm loading-spinner"></span>
			</div>
		{:else if $state.browseDirectories.length === 0}
			<div class="p-4 text-center text-sm opacity-50">No subdirectories</div>
		{:else}
			{#each $state.browseDirectories as entry (entry.path)}
				<div class="flex items-center border-b border-base-200 last:border-b-0">
					<button
						class="btn flex-1 justify-start gap-2 rounded-none font-normal btn-ghost btn-sm"
						onclick={() => handleNavigate(entry.path)}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4 text-warning"
							fill="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2z"
							/>
						</svg>
						<span class="truncate">{entry.name}</span>
					</button>
					<button
						class="btn mr-2 btn-ghost btn-xs"
						onclick={() => handleSelect(entry)}
						title="Select this directory"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
						</svg>
					</button>
				</div>
			{/each}
		{/if}
	</div>
</div>
