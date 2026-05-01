<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { artistsService, type Artist } from '$lib/artists.service';
	import { firkinsService, type Firkin } from '$lib/firkins.service';

	interface Props {
		data: { artist: Artist };
	}

	let { data }: Props = $props();
	const artist = $derived<Artist>(data.artist);

	let editing = $state(false);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

	let editName = $state('');
	/** One role per line — split + trimmed + blank-filtered + deduped on save. */
	let editRoles = $state('');
	let editImageUrl = $state('');

	function startEdit() {
		editName = artist.name;
		editRoles = (artist.roles ?? []).join('\n');
		editImageUrl = artist.imageUrl ?? '';
		editing = true;
		saveError = null;
	}

	function cancelEdit() {
		editing = false;
		saveError = null;
	}

	async function save() {
		if (saving) return;
		const trimmed = editName.trim();
		if (!trimmed) {
			saveError = 'name is required';
			return;
		}
		saving = true;
		saveError = null;
		try {
			const seen = new Set<string>();
			const roles: string[] = [];
			for (const line of editRoles.split('\n')) {
				const t = line.trim();
				if (t.length === 0 || seen.has(t)) continue;
				seen.add(t);
				roles.push(t);
			}
			const updated = await artistsService.update(artist.id, {
				name: trimmed,
				roles,
				imageUrl: editImageUrl.trim() ? editImageUrl.trim() : undefined
			});
			data.artist = updated;
			editing = false;
		} catch (err) {
			saveError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			saving = false;
		}
	}

	async function remove() {
		if (removing) return;
		// PUT-in-place mutation rule: remove by CID. The user may want to
		// keep referencing firkins intact (their `artistIds` will simply
		// resolve to nothing for this CID), so we don't try to scrub them.
		if (
			!window.confirm(
				'Delete this artist record? Firkins that reference its CID will keep the reference but resolve to nothing.'
			)
		) {
			return;
		}
		removing = true;
		removeError = null;
		try {
			await artistsService.remove(artist.id);
			await goto(`${base}/artist`);
		} catch (err) {
			removeError = err instanceof Error ? err.message : 'Unknown error';
			removing = false;
		}
	}

	const firkinsStore = firkinsService.state;
	onMount(() => firkinsService.start());

	const referencingFirkins = $derived<Firkin[]>(
		$firkinsStore.firkins.filter((f) => (f.artistIds ?? []).includes(artist.id))
	);

	function initials(name: string): string {
		return name
			.split(/\s+/)
			.filter((p) => p.length > 0)
			.map((p) => p[0]!.toUpperCase())
			.slice(0, 2)
			.join('');
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
	<title>Mhaol Cloud — {artist.name}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/artist">← Artists</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{artist.name}</h1>
			{#if artist.roles.length > 0}
				<div class="flex flex-wrap gap-1">
					{#each artist.roles as role (role)}
						<span class="badge badge-outline badge-sm">{role}</span>
					{/each}
				</div>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			{#if !editing}
				<button type="button" class="btn btn-outline btn-sm" onclick={startEdit}>Edit</button>
			{/if}
			<button
				type="button"
				class="btn btn-outline btn-sm btn-error"
				onclick={remove}
				disabled={removing}
			>
				{removing ? 'Deleting…' : 'Delete'}
			</button>
		</div>
	</header>

	{#if removeError}
		<div class="alert alert-error">
			<span>{removeError}</span>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<div
				class="flex flex-col items-center gap-2 rounded-box border border-base-content/10 bg-base-200 p-4"
			>
				{#if artist.imageUrl}
					<img
						src={artist.imageUrl}
						alt={artist.name}
						class="h-48 w-48 rounded-full object-cover"
						loading="lazy"
					/>
				{:else}
					<span
						class="flex h-48 w-48 items-center justify-center rounded-full bg-base-300 text-3xl font-semibold text-base-content/60"
					>
						{initials(artist.name)}
					</span>
				{/if}
				<span class="text-center text-sm font-medium [overflow-wrap:anywhere]">{artist.name}</span>
				{#if artist.roles.length > 0}
					<div class="flex flex-wrap justify-center gap-1">
						{#each artist.roles as role (role)}
							<span class="badge badge-ghost badge-xs">{role}</span>
						{/each}
					</div>
				{/if}
			</div>
		</aside>

		<section class="flex flex-col gap-6">
			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Identity</h2>
				<table class="table table-sm">
					<tbody>
						<tr>
							<th class="w-32 align-top">CID</th>
							<td class="font-mono text-xs break-all">{artist.id}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Created</th>
							<td class="text-xs">{formatDate(artist.created_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Updated</th>
							<td class="text-xs">{formatDate(artist.updated_at)}</td>
						</tr>
					</tbody>
				</table>
				<p class="mt-2 text-[10px] text-base-content/50">
					The CID is computed from the (normalised) <code>name</code> only — adding a role or
					updating the image URL does not roll the CID. Different artists with the same name will
					collide; the cloud merges them into one record by design (see
					<code>artists::upsert</code>). To create a distinct record, use a different name.
				</p>
			</div>

			{#if editing}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Edit</h2>
					<div class="flex flex-col gap-2">
						<label class="form-control w-full">
							<span class="text-xs text-base-content/60">Name</span>
							<input
								type="text"
								class="input-bordered input input-sm"
								bind:value={editName}
								disabled={saving}
							/>
						</label>
						<label class="form-control w-full">
							<span class="text-xs text-base-content/60">Roles (one per line)</span>
							<textarea
								class="textarea-bordered textarea font-mono textarea-sm"
								placeholder={'Director\nActor as Forrest Gump\nProducer'}
								rows="4"
								bind:value={editRoles}
								disabled={saving}
							></textarea>
						</label>
						<label class="form-control w-full">
							<span class="text-xs text-base-content/60">Image URL</span>
							<input
								type="text"
								class="input-bordered input input-sm"
								bind:value={editImageUrl}
								disabled={saving}
							/>
						</label>
						{#if saveError}
							<p class="text-xs text-error">{saveError}</p>
						{/if}
						<div class="flex justify-end gap-2 pt-2">
							<button
								type="button"
								class="btn btn-ghost btn-sm"
								onclick={cancelEdit}
								disabled={saving}
							>
								Cancel
							</button>
							<button type="button" class="btn btn-sm btn-primary" onclick={save} disabled={saving}>
								{saving ? 'Saving…' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
					Referenced by{referencingFirkins.length > 0 ? ` (${referencingFirkins.length})` : ''}
				</h2>
				{#if $firkinsStore.loading && referencingFirkins.length === 0}
					<p class="text-sm text-base-content/60">Loading firkins…</p>
				{:else if referencingFirkins.length === 0}
					<p class="text-sm text-base-content/60">
						No firkins reference this artist. Bookmark a catalog item that lists this artist among
						its credits, or add the artist manually from the new-firkin form, to see them here.
					</p>
				{:else}
					<ul class="grid grid-cols-1 gap-2 md:grid-cols-2">
						{#each referencingFirkins as firkin (firkin.id)}
							<li>
								<a
									href="{base}/catalog/{encodeURIComponent(firkin.id)}"
									class="flex items-center gap-3 rounded border border-base-content/10 bg-base-100 p-2 hover:border-base-content/30"
								>
									{#if firkin.images[0]?.url}
										<img
											src={firkin.images[0].url}
											alt={firkin.title}
											class="h-12 w-12 shrink-0 rounded object-cover"
											loading="lazy"
										/>
									{:else}
										<span
											class="flex h-12 w-12 shrink-0 items-center justify-center rounded bg-base-300 text-xs font-semibold text-base-content/60"
										>
											{initials(firkin.title)}
										</span>
									{/if}
									<div class="flex min-w-0 flex-1 flex-col">
										<span class="truncate text-sm font-medium">{firkin.title}</span>
										<span class="truncate text-xs text-base-content/60">{firkin.addon}</span>
									</div>
								</a>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</section>
	</div>
</div>
