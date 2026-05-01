<script lang="ts">
	import classNames from 'classnames';
	import { ed2kService } from 'ui-lib/services/ed2k.service';

	const ed2kState = ed2kService.state;

	let linkInput = $state('');
	let adding = $state(false);

	let isEd2kLink = $derived(linkInput.startsWith('ed2k://|file|'));
	let canAdd = $derived(linkInput.trim() && isEd2kLink && !adding && $ed2kState.initialized);

	async function handleAdd() {
		if (!canAdd) return;
		adding = true;
		const result = await ed2kService.addFile(linkInput.trim());
		adding = false;
		if (result) linkInput = '';
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && canAdd) handleAdd();
	}
</script>

<div class="form-control">
	<div class="join w-full">
		<input
			type="text"
			bind:value={linkInput}
			onkeydown={handleKeydown}
			placeholder="ed2k://|file|name|size|hash|/"
			class={classNames('input-bordered input join-item flex-1', {
				'input-error': linkInput && !isEd2kLink,
				'input-success': isEd2kLink && linkInput
			})}
			disabled={!$ed2kState.initialized}
		/>
		<button class="btn join-item btn-primary" onclick={handleAdd} disabled={!canAdd}>
			{#if adding}
				<span class="loading loading-sm loading-spinner"></span>
			{:else}
				Add
			{/if}
		</button>
	</div>
	{#if linkInput && !isEd2kLink}
		<span class="label">
			<span class="label-text-alt text-error">Enter an ed2k:// file link</span>
		</span>
	{/if}
</div>
