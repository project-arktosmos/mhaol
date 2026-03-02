<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';
	import type { Identity } from '$types/identity.type';
	import { identityService } from '$services/identity.service';

	let identities = $state<Identity[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let newName = $state('');
	let creating = $state(false);

	let confirmTarget = $state<{ name: string; action: 'regenerate' | 'delete' } | null>(null);
	let actionLoading = $state(false);

	let copiedAddress = $state<string | null>(null);
	let copiedPassport = $state<string | null>(null);

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
				throw new Error(
					(body as { message?: string }).message ?? `HTTP ${res.status}`
				);
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

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Identities</h1>
		<p class="text-sm opacity-70">
			Ethereum identities stored server-side. Private keys never leave the server.
		</p>
	</div>

	{#if error}
		<div class="alert alert-error mb-4">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	<!-- Create form -->
	<div class="mb-6 flex items-end gap-2">
		<div class="form-control flex-1">
			<label class="label" for="new-identity-name">
				<span class="label-text">New identity name</span>
			</label>
			<input
				id="new-identity-name"
				type="text"
				class="input input-bordered input-sm"
				placeholder="e.g. MY_WALLET"
				bind:value={newName}
				disabled={creating}
				onkeydown={(e) => e.key === 'Enter' && createIdentity()}
			/>
		</div>
		<button
			class={classNames('btn btn-primary btn-sm', { loading: creating })}
			disabled={creating || !newName.trim()}
			onclick={createIdentity}
		>
			{#if creating}
				<span class="loading loading-spinner loading-xs"></span>
			{:else}
				Create
			{/if}
		</button>
	</div>

	<!-- Identity list -->
	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if identities.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No identities yet. Create one above.</p>
		</div>
	{:else}
		<div class="flex flex-col gap-3">
			{#each identities as identity (identity.name)}
				<div class="card bg-base-200">
					<div class="card-body gap-3 p-4">
						<div class="flex items-center gap-4">
							<div class="flex-1 min-w-0">
								<div class="font-mono text-sm font-semibold">{identity.name}</div>
								<div class="flex items-center gap-2 mt-1">
									<code class="break-all text-xs opacity-70">
										{identity.address}
									</code>
									<button
										class="btn btn-ghost btn-xs h-auto min-h-0 px-1 py-0.5"
										title="Copy address"
										onclick={() => copyAddress(identity.address)}
									>
										{copiedAddress === identity.address ? 'Copied!' : 'Copy'}
									</button>
								</div>
							</div>
							<div class="flex gap-1">
								<button
									class="btn btn-ghost btn-xs"
									onclick={() =>
										(confirmTarget = { name: identity.name, action: 'regenerate' })}
								>
									Regenerate
								</button>
								<button
									class="btn btn-ghost btn-xs text-error"
									onclick={() =>
										(confirmTarget = { name: identity.name, action: 'delete' })}
								>
									Delete
								</button>
							</div>
						</div>

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
								class="textarea textarea-bordered w-full font-mono text-xs"
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
</div>

{#if confirmTarget}
	<div class="modal modal-open">
		<div class="modal-box">
			{#if confirmTarget.action === 'regenerate'}
				<h3 class="text-lg font-bold">Regenerate "{confirmTarget.name}"?</h3>
				<p class="py-4">
					This will permanently discard the current private key and generate a new one.
					The address will change and peers using it will no longer be able to reach you.
				</p>
			{:else}
				<h3 class="text-lg font-bold">Delete "{confirmTarget.name}"?</h3>
				<p class="py-4">
					This will permanently remove this identity. The private key will be lost.
				</p>
			{/if}
			<div class="modal-action">
				<button
					class="btn btn-ghost"
					disabled={actionLoading}
					onclick={() => (confirmTarget = null)}
				>
					Cancel
				</button>
				<button
					class="btn btn-error"
					disabled={actionLoading}
					onclick={handleConfirm}
				>
					{#if actionLoading}
						<span class="loading loading-spinner loading-xs"></span>
					{:else}
						{confirmTarget.action === 'regenerate' ? 'Regenerate' : 'Delete'}
					{/if}
				</button>
			</div>
		</div>
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<div class="modal-backdrop" onclick={() => (confirmTarget = null)}></div>
	</div>
{/if}
