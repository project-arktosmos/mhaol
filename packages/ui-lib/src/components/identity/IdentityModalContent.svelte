<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import type { Identity } from 'ui-lib/types/identity.type';
	import { identityService } from 'ui-lib/services/identity.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';

	let identities = $state<Identity[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let newName = $state('');
	let creating = $state(false);

	let confirmTarget = $state<{ name: string; action: 'regenerate' | 'delete' } | null>(null);
	let actionLoading = $state(false);

	let copiedAddress = $state<string | null>(null);
	let copiedPassport = $state<string | null>(null);

	let editingProfile = $state<string | null>(null);
	let editUsername = $state('');
	let editProfilePicture = $state('');
	let savingProfile = $state(false);

	onMount(() => {
		loadIdentities();
	});

	async function loadIdentities() {
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl('/api/identities'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			identities = await res.json();
			await identityService.refresh();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function createIdentity() {
		if (!newName.trim()) return;
		creating = true;
		error = null;
		try {
			const res = await fetch(apiUrl('/api/identities'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name: newName.trim() })
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { message?: string }).message ?? `HTTP ${res.status}`);
			}
			newName = '';
			await loadIdentities();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			creating = false;
		}
	}

	async function handleConfirm() {
		if (!confirmTarget) return;
		actionLoading = true;
		error = null;

		const { name, action } = confirmTarget;
		try {
			if (action === 'regenerate') {
				const res = await fetch(apiUrl(`/api/identities/${encodeURIComponent(name)}`), {
					method: 'PUT'
				});
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
			} else {
				const res = await fetch(apiUrl(`/api/identities/${encodeURIComponent(name)}`), {
					method: 'DELETE'
				});
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
			}
			confirmTarget = null;
			await loadIdentities();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionLoading = false;
		}
	}

	async function copyAddress(address: string) {
		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(address);
			} else {
				const textarea = document.createElement('textarea');
				textarea.value = address;
				textarea.style.position = 'fixed';
				textarea.style.opacity = '0';
				document.body.appendChild(textarea);
				textarea.select();
				document.execCommand('copy');
				document.body.removeChild(textarea);
			}
			copiedAddress = address;
			setTimeout(() => (copiedAddress = null), 1500);
		} catch {
			// Copy failed silently
		}
	}

	function shortAddress(addr: string): string {
		if (!addr.startsWith('0x') || addr.length < 10) return addr;
		return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
	}

	function prettyJson(jsonStr: string): string {
		try {
			return JSON.stringify(JSON.parse(jsonStr), null, 2);
		} catch {
			return jsonStr;
		}
	}

	function downloadPassport(passport: string, address: string) {
		const blob = new Blob([prettyJson(passport)], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `${address}.passport.json`;
		a.click();
		URL.revokeObjectURL(url);
	}

	function startEditProfile(identity: Identity) {
		editingProfile = identity.name;
		editUsername = identity.username ?? '';
		editProfilePicture = identity.profilePictureUrl ?? '';
	}

	async function saveProfile() {
		if (!editingProfile) return;
		savingProfile = true;
		error = null;
		try {
			const body: Record<string, string> = { username: editUsername };
			if (editProfilePicture) body.profilePictureUrl = editProfilePicture;
			const res = await fetch(
				apiUrl(`/api/identities/${encodeURIComponent(editingProfile)}/profile`),
				{
					method: 'PATCH',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify(body)
				}
			);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			editingProfile = null;
			await loadIdentities();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			savingProfile = false;
		}
	}

	async function copyPassport(passport: string) {
		try {
			const text = prettyJson(passport);
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(text);
			} else {
				const el = document.createElement('textarea');
				el.value = text;
				el.style.position = 'fixed';
				el.style.opacity = '0';
				document.body.appendChild(el);
				el.select();
				document.execCommand('copy');
				document.body.removeChild(el);
			}
			copiedPassport = passport;
			setTimeout(() => (copiedPassport = null), 1500);
		} catch {
			// Copy failed silently
		}
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Identities</h3>
	<p class="text-sm text-base-content/60">
		Ethereum identities stored server-side. Private keys never leave the server.
	</p>
</div>

{#if error}
	<div class="mt-4 alert alert-error">
		<span>{error}</span>
		<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
	</div>
{/if}

<!-- Create form -->
<div class="mt-4 flex items-end gap-2">
	<div class="form-control flex-1">
		<label class="label" for="new-identity-name">
			<span class="label-text">New identity name</span>
		</label>
		<input
			id="new-identity-name"
			type="text"
			class="input-bordered input input-sm"
			placeholder="e.g. MY_WALLET"
			bind:value={newName}
			disabled={creating}
			onkeydown={(e) => e.key === 'Enter' && createIdentity()}
		/>
	</div>
	<button
		class={classNames('btn btn-sm btn-primary', { loading: creating })}
		disabled={creating || !newName.trim()}
		onclick={createIdentity}
	>
		{#if creating}
			<span class="loading loading-xs loading-spinner"></span>
		{:else}
			Create
		{/if}
	</button>
</div>

<!-- Identity list -->
{#if loading}
	<div class="flex justify-center py-12">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if identities.length === 0}
	<div class="mt-4 rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No identities yet. Create one above.</p>
	</div>
{:else}
	<div class="mt-4 flex flex-col gap-3">
		{#each identities as identity (identity.name)}
			<div class="card bg-base-200">
				<div class="card-body gap-3 p-4">
					<div class="flex items-center gap-4">
						<div class="min-w-0 flex-1">
							<div class="font-mono text-sm font-semibold">{identity.name}</div>
							{#if identity.username}
								<div class="text-xs opacity-60">{identity.username}</div>
							{/if}
							<div class="mt-1 flex items-center gap-2">
								<code class="text-xs break-all opacity-70">
									{identity.address}
								</code>
								<button
									class="btn h-auto min-h-0 px-1 py-0.5 btn-ghost btn-xs"
									title="Copy address"
									onclick={() => copyAddress(identity.address)}
								>
									{copiedAddress === identity.address ? 'Copied!' : 'Copy'}
								</button>
							</div>
						</div>
						<div class="flex gap-1">
							<button class="btn btn-ghost btn-xs" onclick={() => startEditProfile(identity)}>
								Profile
							</button>
							<button
								class="btn btn-ghost btn-xs"
								onclick={() => (confirmTarget = { name: identity.name, action: 'regenerate' })}
							>
								Regenerate
							</button>
							<button
								class="btn text-error btn-ghost btn-xs"
								onclick={() => (confirmTarget = { name: identity.name, action: 'delete' })}
							>
								Delete
							</button>
						</div>
					</div>

					{#if editingProfile === identity.name}
						<div class="flex flex-col gap-2 rounded-lg bg-base-300 p-3">
							<input
								type="text"
								class="input-bordered input input-sm w-full"
								placeholder="Username"
								bind:value={editUsername}
							/>
							<input
								type="url"
								class="input-bordered input input-sm w-full"
								placeholder="Profile picture URL"
								bind:value={editProfilePicture}
							/>
							<div class="flex justify-end gap-1">
								<button class="btn btn-ghost btn-xs" onclick={() => (editingProfile = null)}
									>Cancel</button
								>
								<button
									class={classNames('btn btn-xs btn-primary', { loading: savingProfile })}
									disabled={savingProfile || !editUsername.trim()}
									onclick={saveProfile}
								>
									{#if savingProfile}
										<span class="loading loading-xs loading-spinner"></span>
									{:else}
										Save
									{/if}
								</button>
							</div>
						</div>
					{/if}

					<div>
						<div class="flex items-center justify-between">
							<label class="label py-0">
								<span class="label-text text-xs opacity-50">Passport</span>
							</label>
							<div class="flex gap-1">
								<button
									class="btn btn-ghost btn-xs"
									title="Copy passport"
									onclick={() => copyPassport(identity.passport)}
								>
									{copiedPassport === identity.passport ? 'Copied!' : 'Copy'}
								</button>
								<button
									class="btn btn-ghost btn-xs"
									title="Download passport"
									onclick={() => downloadPassport(identity.passport, identity.address)}
								>
									Download
								</button>
							</div>
						</div>
						<textarea
							class="textarea-bordered textarea w-full font-mono text-xs"
							rows="7"
							disabled
							value={prettyJson(identity.passport)}
						></textarea>
					</div>
				</div>
			</div>
		{/each}
	</div>
{/if}

<Modal open={!!confirmTarget} zIndex={60} onclose={() => (confirmTarget = null)}>
	{#if confirmTarget}
		{#if confirmTarget.action === 'regenerate'}
			<h3 class="text-lg font-bold">Regenerate "{confirmTarget.name}"?</h3>
			<p class="py-4">
				This will permanently discard the current private key and generate a new one. The address
				will change and peers using it will no longer be able to reach you.
			</p>
		{:else}
			<h3 class="text-lg font-bold">Delete "{confirmTarget.name}"?</h3>
			<p class="py-4">This will permanently remove this identity. The private key will be lost.</p>
		{/if}
		<div class="modal-action">
			<button class="btn btn-ghost" disabled={actionLoading} onclick={() => (confirmTarget = null)}>
				Cancel
			</button>
			<button class="btn btn-error" disabled={actionLoading} onclick={handleConfirm}>
				{#if actionLoading}
					<span class="loading loading-xs loading-spinner"></span>
				{:else}
					{confirmTarget.action === 'regenerate' ? 'Regenerate' : 'Delete'}
				{/if}
			</button>
		</div>
	{/if}
</Modal>
