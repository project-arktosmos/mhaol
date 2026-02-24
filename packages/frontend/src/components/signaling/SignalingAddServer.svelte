<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{
		add: { name: string; url: string };
		cancel: void;
	}>();

	let name = '';
	let url = '';
	let urlError = '';

	function validateUrl(value: string): boolean {
		try {
			const parsed = new URL(value);
			return parsed.protocol === 'http:' || parsed.protocol === 'https:';
		} catch {
			return false;
		}
	}

	function handleSubmit() {
		urlError = '';
		if (!name.trim()) return;
		if (!validateUrl(url.trim())) {
			urlError = 'Enter a valid HTTP or HTTPS URL';
			return;
		}
		dispatch('add', { name: name.trim(), url: url.trim() });
	}

	function handleCancel() {
		dispatch('cancel');
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-base">Add Signaling Server</h2>

		<div class="flex flex-col gap-3">
			<div class="flex flex-col gap-1">
				<label class="label py-0" for="server-name">
					<span class="label-text text-xs font-medium">Name</span>
				</label>
				<input
					id="server-name"
					class="input input-bordered input-sm"
					type="text"
					placeholder="My Signaling Server"
					bind:value={name}
				/>
			</div>

			<div class="flex flex-col gap-1">
				<label class="label py-0" for="server-url">
					<span class="label-text text-xs font-medium">URL</span>
				</label>
				<input
					id="server-url"
					class="input input-bordered input-sm"
					class:input-error={urlError}
					type="url"
					placeholder="http://localhost:3002"
					bind:value={url}
				/>
				{#if urlError}
					<span class="text-xs text-error">{urlError}</span>
				{/if}
			</div>
		</div>

		<div class="card-actions justify-end">
			<button class="btn btn-ghost btn-sm" on:click={handleCancel}>Cancel</button>
			<button
				class="btn btn-primary btn-sm"
				on:click={handleSubmit}
				disabled={!name.trim() || !url.trim()}
			>
				Add Server
			</button>
		</div>
	</div>
</div>
