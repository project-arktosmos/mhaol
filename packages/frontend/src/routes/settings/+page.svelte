<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { getApiBase, getDefaultApiBase, setApiBase, subscribeApiBase } from '$lib/api-base';

	let currentBase = $state('');
	let defaultBase = $state('');
	let draft = $state('');
	let saving = $state(false);
	let savedAt = $state<number | null>(null);
	let probing = $state(false);
	let probeResult = $state<{ ok: boolean; message: string } | null>(null);

	onMount(() => {
		currentBase = getApiBase();
		defaultBase = getDefaultApiBase();
		draft = currentBase;
		const off = subscribeApiBase((next) => {
			currentBase = next;
		});
		return off;
	});

	const dirty = $derived(draft.trim().replace(/\/+$/, '') !== currentBase);
	const isDefault = $derived(currentBase === defaultBase);

	function save(): void {
		saving = true;
		try {
			setApiBase(draft);
			savedAt = Date.now();
		} finally {
			saving = false;
		}
	}

	function resetToDefault(): void {
		draft = '';
		setApiBase('');
		savedAt = Date.now();
	}

	async function probe(): Promise<void> {
		probing = true;
		probeResult = null;
		const target = (draft.trim().replace(/\/+$/, '') || defaultBase) + '/api/cloud/status';
		try {
			const res = await fetch(target, { cache: 'no-store' });
			if (!res.ok) {
				probeResult = { ok: false, message: `HTTP ${res.status} ${res.statusText}` };
				return;
			}
			const body = (await res.json()) as { status?: string; version?: string };
			probeResult = {
				ok: true,
				message: `Reached backend (${body.status ?? 'ok'}${body.version ? `, v${body.version}` : ''})`
			};
		} catch (err) {
			probeResult = {
				ok: false,
				message: err instanceof Error ? err.message : 'Connection failed'
			};
		} finally {
			probing = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Settings</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header>
		<h1 class="text-2xl font-bold">Settings</h1>
		<p class="text-sm text-base-content/60">
			Configure how this client connects to the Mhaol backend.
		</p>
	</header>

	<section class="card bg-base-200 p-4">
		<h2 class="mb-2 text-lg font-semibold">Backend URL</h2>
		<p class="mb-3 text-sm text-base-content/60">
			The base URL of the cloud server this client should talk to. Leave blank to fall back to the
			built-in default for this build.
		</p>

		<dl class="mb-4 grid grid-cols-1 gap-2 text-sm sm:grid-cols-[max-content_1fr] sm:gap-x-4">
			<dt class="text-base-content/60">Active</dt>
			<dd class="font-mono break-all">
				{currentBase || '(same-origin)'}
				{#if isDefault}
					<span class="ml-2 badge badge-ghost badge-sm">default</span>
				{:else}
					<span class="ml-2 badge badge-sm badge-info">override</span>
				{/if}
			</dd>
			<dt class="text-base-content/60">Default for this build</dt>
			<dd class="font-mono break-all">{defaultBase || '(same-origin)'}</dd>
		</dl>

		<label class="form-control w-full max-w-xl">
			<div class="label">
				<span class="label-text">Override URL</span>
			</div>
			<input
				type="url"
				class="input-bordered input"
				placeholder={defaultBase || 'http://localhost:9898'}
				bind:value={draft}
				autocomplete="off"
				spellcheck="false"
			/>
			<div class="label">
				<span class="label-text-alt text-base-content/60">
					Examples: <code>http://127.0.0.1:9898</code> ·
					<code>http://192.168.1.50:9898</code> · <code>https://mhaol.example.com</code>
				</span>
			</div>
		</label>

		<div class="mt-3 flex flex-wrap gap-2">
			<button class="btn btn-sm btn-primary" disabled={!dirty || saving} onclick={save}>
				{saving ? 'Saving…' : 'Save'}
			</button>
			<button class="btn btn-outline btn-sm" disabled={probing} onclick={probe}>
				{probing ? 'Checking…' : 'Test connection'}
			</button>
			<button class="btn btn-outline btn-sm" disabled={isDefault} onclick={resetToDefault}>
				Reset to default
			</button>
		</div>

		{#if savedAt}
			<p class="mt-3 text-xs text-base-content/60">
				Saved. Reload the page for every service to pick up the new backend.
			</p>
		{/if}
		{#if probeResult}
			<p
				class={classNames('mt-3 text-sm', {
					'text-success': probeResult.ok,
					'text-error': !probeResult.ok
				})}
			>
				{probeResult.message}
			</p>
		{/if}
	</section>
</div>
