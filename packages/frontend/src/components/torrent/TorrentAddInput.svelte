<script lang="ts">
	import classNames from 'classnames';
	import { torrentService } from '$services/torrent.service';

	const state = torrentService.state;

	let magnetInput = '';
	let adding = false;

	$: isMagnet = magnetInput.startsWith('magnet:');
	$: isUrl =
		magnetInput.startsWith('http://') || magnetInput.startsWith('https://');
	$: isValidSource = isMagnet || isUrl || magnetInput.endsWith('.torrent');
	$: canAdd = magnetInput.trim() && isValidSource && !adding && $state.initialized;

	async function handleAdd() {
		if (!canAdd) return;

		adding = true;
		const result = await torrentService.addTorrent(magnetInput.trim());
		adding = false;

		if (result) {
			magnetInput = '';
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && canAdd) {
			handleAdd();
		}
	}

	function handlePaste() {
		setTimeout(() => {
			if (isValidSource && $state.initialized) {
				handleAdd();
			}
		}, 100);
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Add Torrent</h2>

		<div class="form-control">
			<div class="join w-full">
				<input
					type="text"
					bind:value={magnetInput}
					on:keydown={handleKeydown}
					on:paste={handlePaste}
					placeholder="magnet:?xt=urn:btih:... or torrent URL"
					class={classNames('input join-item input-bordered flex-1', {
						'input-error': magnetInput && !isValidSource,
						'input-success': isValidSource && magnetInput
					})}
					disabled={!$state.initialized}
				/>
				<button
					class="btn btn-primary join-item"
					on:click={handleAdd}
					disabled={!canAdd}
				>
					{#if adding}
						<span class="loading loading-spinner loading-sm"></span>
					{:else}
						Add
					{/if}
				</button>
			</div>
			{#if magnetInput && !isValidSource}
				<span class="label">
					<span class="label-text-alt text-error"
						>Enter a magnet link, torrent URL, or .torrent file path</span
					>
				</span>
			{/if}
		</div>

		<p class="text-xs text-base-content/50">
			Paste a magnet link or torrent URL to start downloading.
		</p>
	</div>
</div>
