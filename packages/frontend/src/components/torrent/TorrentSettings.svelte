<script lang="ts">
	import classNames from 'classnames';
	import { torrentService } from '$services/torrent.service';
	import LibrarySelector from '$components/libraries/LibrarySelector.svelte';
	import ConnectionStatus from '$components/core/ConnectionStatus.svelte';

	const torrentState = torrentService.state;

	// Clear storage confirmation
	let confirmClear = $state(false);

	function handleLibrarySelect(libraryId: string) {
		torrentService.setLibrary(libraryId);
	}

	async function handleClearStorage() {
		if (!confirmClear) {
			confirmClear = true;
			return;
		}
		await torrentService.clearStorage();
		confirmClear = false;
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Settings</h2>

		<!-- Connection Status -->
		<ConnectionStatus connected={$torrentState.initialized} />

		<!-- Download Library -->
		<LibrarySelector currentLibraryId={$torrentState.libraryId} onselect={handleLibrarySelect} />

		<!-- Clear Storage -->
		<div class="form-control">
			<button
				class={classNames('btn btn-sm', {
					'btn-error': confirmClear,
					'btn-outline': !confirmClear
				})}
				onclick={handleClearStorage}
			>
				{#if confirmClear}
					Confirm Clear Storage
				{:else}
					Clear Storage
				{/if}
			</button>
			{#if confirmClear}
				<span class="label">
					<span class="label-text-alt text-warning"
						>This will delete all downloaded files and persistence data</span
					>
				</span>
			{/if}
		</div>
	</div>
</div>
