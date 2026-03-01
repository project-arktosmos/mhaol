<script lang="ts">
	import { onMount } from 'svelte';
	import { downloadsService } from '$services/downloads.service';

	onMount(() => {
		downloadsService.openModal();
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Downloads</h1>
		<p class="text-sm opacity-70">All YouTube and torrent downloads</p>
	</div>

	<button class="btn btn-primary" onclick={() => downloadsService.openModal()}>
		Open Downloads
	</button>
</div>
