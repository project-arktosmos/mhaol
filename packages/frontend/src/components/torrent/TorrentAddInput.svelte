<script lang="ts">
	import classNames from 'classnames';
	import { torrentService } from '$services/torrent.service';

	const torrentState = torrentService.state;

	let magnetInput = $state('');
	let adding = $state(false);

	let isMagnet = $derived(magnetInput.startsWith('magnet:'));
	let isUrl = $derived(magnetInput.startsWith('http://') || magnetInput.startsWith('https://'));
	let isValidSource = $derived(isMagnet || isUrl || magnetInput.endsWith('.torrent'));
	let canAdd = $derived(
		magnetInput.trim() && isValidSource && !adding && $torrentState.initialized
	);

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
			if (isValidSource && $torrentState.initialized) {
				handleAdd();
			}
		}, 100);
	}
</script>

<div class="form-control">
	<div class="join w-full">
		<input
			type="text"
			bind:value={magnetInput}
			onkeydown={handleKeydown}
			onpaste={handlePaste}
			placeholder="magnet:?xt=urn:btih:... or torrent URL"
			class={classNames('input-bordered input join-item flex-1', {
				'input-error': magnetInput && !isValidSource,
				'input-success': isValidSource && magnetInput
			})}
			disabled={!$torrentState.initialized}
		/>
		<button class="btn join-item btn-primary" onclick={handleAdd} disabled={!canAdd}>
			{#if adding}
				<span class="loading loading-sm loading-spinner"></span>
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
