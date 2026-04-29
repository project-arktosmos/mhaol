<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { documentsService } from '$lib/documents.service';

	const docsStore = documentsService.state;

	let name = $state('');
	let author = $state('');
	let description = $state('');
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let deletingId = $state<string | null>(null);

	onMount(() => {
		documentsService.refresh();
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		createError = null;
		const trimmedName = name.trim();
		const trimmedAuthor = author.trim();
		if (!trimmedName) {
			createError = 'Name is required';
			return;
		}
		if (!trimmedAuthor) {
			createError = 'Author is required';
			return;
		}
		creating = true;
		try {
			await documentsService.create(trimmedName, trimmedAuthor, description.trim());
			name = '';
			author = '';
			description = '';
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			creating = false;
		}
	}

	async function remove(id: string) {
		deletingId = id;
		try {
			await documentsService.remove(id);
		} catch (err) {
			documentsService.state.update((s) => ({
				...s,
				error: err instanceof Error ? err.message : 'Unknown error'
			}));
		} finally {
			deletingId = null;
		}
	}

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Documents</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Documents</h1>
			<p class="text-sm text-base-content/60">
				Documents stored in the cloud's SurrealDB. Each entry has a name, an author, and a
				description.
			</p>
		</div>
		<button
			class="btn btn-outline btn-sm"
			onclick={() => documentsService.refresh()}
			disabled={$docsStore.loading}
		>
			Refresh
		</button>
	</header>

	{#if $docsStore.error}
		<div class="alert alert-error">
			<span>{$docsStore.error}</span>
		</div>
	{/if}

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-3 text-lg font-semibold">Add a document</h2>
		<form class="flex flex-col gap-3" onsubmit={submit}>
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<tbody>
						<tr>
							<th class="w-32 align-middle">Name</th>
							<td>
								<input
									type="text"
									class="input-bordered input input-sm w-full"
									placeholder="Project brief"
									bind:value={name}
									disabled={creating}
								/>
							</td>
						</tr>
						<tr>
							<th class="w-32 align-middle">Author</th>
							<td>
								<input
									type="text"
									class="input-bordered input input-sm w-full"
									placeholder="Jane Doe"
									bind:value={author}
									disabled={creating}
								/>
							</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Description</th>
							<td>
								<input
									type="text"
									class="input-bordered input input-sm w-full"
									placeholder="Short summary of the document"
									bind:value={description}
									disabled={creating}
								/>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
			<div>
				<button
					type="submit"
					class={classNames('btn btn-sm btn-primary', { 'btn-disabled': creating })}
					disabled={creating}
				>
					{creating ? 'Creating…' : 'Create'}
				</button>
			</div>
		</form>
		{#if createError}
			<p class="mt-2 text-sm text-error">{createError}</p>
		{/if}
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Existing documents</h2>
		{#if $docsStore.loading && $docsStore.documents.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $docsStore.documents.length === 0}
			<p class="text-sm text-base-content/60">No documents yet.</p>
		{:else}
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<thead>
						<tr>
							<th>Name</th>
							<th>Author</th>
							<th>Description</th>
							<th>Created</th>
							<th class="w-24"></th>
						</tr>
					</thead>
					<tbody>
						{#each $docsStore.documents as doc (doc.id)}
							<tr>
								<td class="font-medium">{doc.name}</td>
								<td>{doc.author}</td>
								<td class="max-w-md text-xs whitespace-pre-wrap text-base-content/80"
									>{doc.description}</td
								>
								<td class="text-xs text-base-content/60">{formatDate(doc.created_at)}</td>
								<td class="text-right">
									<button
										class="btn text-error btn-ghost btn-xs"
										onclick={() => remove(doc.id)}
										disabled={deletingId === doc.id}
									>
										{deletingId === doc.id ? 'Removing…' : 'Remove'}
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>
