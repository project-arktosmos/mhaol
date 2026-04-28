<script lang="ts">
	import { onMount } from 'svelte';
	import { ed2kService } from 'ui-lib/services/ed2k.service';
	import Ed2kAddInput from 'ui-lib/components/ed2k/Ed2kAddInput.svelte';
	import Ed2kSearch from 'ui-lib/components/ed2k/Ed2kSearch.svelte';
	import Ed2kStatusBar from 'ui-lib/components/ed2k/Ed2kStatusBar.svelte';
	import Ed2kFileList from 'ui-lib/components/ed2k/Ed2kFileList.svelte';

	const state = ed2kService.state;

	onMount(() => {
		ed2kService.initialize();
	});
</script>

<div class="flex flex-col gap-4">
	{#if $state.error}
		<div class="alert alert-warning">
			<span>{$state.error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => ed2kService.dismissError()}>
				Dismiss
			</button>
		</div>
	{/if}

	<Ed2kStatusBar />
	<Ed2kAddInput />
	<Ed2kSearch />
	<Ed2kFileList />
</div>
