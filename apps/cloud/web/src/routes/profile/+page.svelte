<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { userIdentityService, USERNAME_PATTERN } from '$lib/user-identity.service';

	const identityStore = userIdentityService.state;

	let importText = $state('');
	let usernameDraft = $state('');
	let savingUsername = $state(false);
	let usernameError = $state<string | null>(null);
	let importError = $state<string | null>(null);
	let actionError = $state<string | null>(null);
	let copied = $state(false);
	let regenerating = $state(false);

	$effect(() => {
		const u = $identityStore.identity?.username;
		if (u !== undefined && usernameDraft === '') usernameDraft = u;
	});

	const exportText = $derived.by(() => {
		// Re-read whenever identity changes.
		void $identityStore.identity;
		try {
			return userIdentityService.exportJson();
		} catch {
			return '';
		}
	});

	const usernameDirty = $derived(
		!!$identityStore.identity && usernameDraft.trim() !== $identityStore.identity.username
	);

	const usernameValid = $derived.by(() => {
		const v = usernameDraft.trim();
		return v.length >= 1 && v.length <= 32 && USERNAME_PATTERN.test(v);
	});

	onMount(() => {
		userIdentityService.initialize();
	});

	async function copyJson(): Promise<void> {
		actionError = null;
		try {
			await navigator.clipboard.writeText(exportText);
			copied = true;
			setTimeout(() => (copied = false), 1500);
		} catch (err) {
			actionError = err instanceof Error ? err.message : 'Copy failed';
		}
	}

	function downloadJson(): void {
		actionError = null;
		try {
			const blob = new Blob([exportText], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `mhaol-cloud-identity-${$identityStore.identity?.address ?? 'unknown'}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
		} catch (err) {
			actionError = err instanceof Error ? err.message : 'Download failed';
		}
	}

	async function importFromText(): Promise<void> {
		importError = null;
		const trimmed = importText.trim();
		if (!trimmed) {
			importError = 'Paste a JSON identity above first';
			return;
		}
		try {
			await userIdentityService.importJson(trimmed);
			importText = '';
			usernameDraft = '';
		} catch (err) {
			importError = err instanceof Error ? err.message : 'Import failed';
		}
	}

	async function importFromFile(event: Event): Promise<void> {
		importError = null;
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		try {
			const text = await file.text();
			await userIdentityService.importJson(text);
			usernameDraft = '';
		} catch (err) {
			importError = err instanceof Error ? err.message : 'Import failed';
		} finally {
			input.value = '';
		}
	}

	async function saveUsername(): Promise<void> {
		usernameError = null;
		savingUsername = true;
		try {
			await userIdentityService.updateUsername(usernameDraft.trim());
		} catch (err) {
			usernameError = err instanceof Error ? err.message : 'Save failed';
		} finally {
			savingUsername = false;
		}
	}

	async function regenerate(): Promise<void> {
		actionError = null;
		const ok = window.confirm(
			'Generate a brand new identity? This will overwrite your current one — export it first if you want to keep it.'
		);
		if (!ok) return;
		regenerating = true;
		try {
			await userIdentityService.regenerate();
			usernameDraft = '';
		} catch (err) {
			actionError = err instanceof Error ? err.message : 'Regenerate failed';
		} finally {
			regenerating = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Profile</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-baseline justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Profile</h1>
			<p class="text-sm text-base-content/60">
				Your identity for this cloud. Stored only in your browser; export the JSON if you want to
				use it on another device.
			</p>
		</div>
	</header>

	{#if $identityStore.loading && !$identityStore.identity}
		<div class="alert">
			<span>Loading identity…</span>
		</div>
	{:else if $identityStore.error}
		<div class="alert alert-error">
			<span>{$identityStore.error}</span>
			<button class="btn btn-sm" onclick={() => userIdentityService.initialize()}>Retry</button>
		</div>
	{/if}

	{#if $identityStore.identity}
		<section class="card bg-base-200 p-4">
			<h2 class="mb-2 text-lg font-semibold">Current identity</h2>
			<dl class="grid grid-cols-1 gap-3 sm:grid-cols-[max-content_1fr] sm:gap-x-4">
				<dt class="text-sm text-base-content/60">Address</dt>
				<dd class="font-mono text-sm break-all">{$identityStore.identity.address}</dd>
				<dt class="text-sm text-base-content/60 sm:pt-2">Username</dt>
				<dd>
					<div class="flex flex-wrap items-start gap-2">
						<input
							type="text"
							class={classNames('input-bordered input input-sm w-full max-w-sm', {
								'input-error': !usernameValid && usernameDraft.length > 0
							})}
							bind:value={usernameDraft}
							maxlength="32"
							autocomplete="off"
						/>
						<button
							class="btn btn-sm btn-primary"
							disabled={!usernameDirty || !usernameValid || savingUsername}
							onclick={saveUsername}
						>
							{savingUsername ? 'Saving…' : 'Save'}
						</button>
					</div>
					<p class="mt-1 text-xs text-base-content/60">
						Letters, digits, and "-" only. 1–32 characters.
					</p>
					{#if usernameError}
						<p class="mt-1 text-xs text-error">{usernameError}</p>
					{/if}
				</dd>
				{#if $identityStore.user}
					<dt class="text-sm text-base-content/60">Registered</dt>
					<dd class="text-sm">
						{new Date($identityStore.user.created_at).toLocaleString()}
					</dd>
					<dt class="text-sm text-base-content/60">Last login</dt>
					<dd class="text-sm">
						{$identityStore.user.last_login_at
							? new Date($identityStore.user.last_login_at).toLocaleString()
							: '—'}
					</dd>
				{/if}
			</dl>
		</section>

		<section class="card bg-base-200 p-4">
			<h2 class="mb-2 text-lg font-semibold">Export</h2>
			<p class="mb-3 text-sm text-base-content/60">
				Save the JSON below to move this identity to another device. It contains your private key —
				keep it secret.
			</p>
			<textarea
				class="textarea-bordered textarea h-40 w-full font-mono text-xs"
				readonly
				value={exportText}
			></textarea>
			<div class="mt-3 flex flex-wrap gap-2">
				<button class="btn btn-outline btn-sm" onclick={copyJson}>
					{copied ? 'Copied!' : 'Copy to clipboard'}
				</button>
				<button class="btn btn-outline btn-sm" onclick={downloadJson}>Download .json</button>
			</div>
		</section>

		<section class="card bg-base-200 p-4">
			<h2 class="mb-2 text-lg font-semibold">Import</h2>
			<p class="mb-3 text-sm text-base-content/60">
				Replace the current identity with one you exported earlier.
			</p>
			<textarea
				class="textarea-bordered textarea h-32 w-full font-mono text-xs"
				placeholder={`{ "address": "0x…", "privateKey": "0x…", "username": "…" }`}
				bind:value={importText}
			></textarea>
			<div class="mt-3 flex flex-wrap items-center gap-2">
				<button class="btn btn-sm btn-primary" onclick={importFromText}>Import from text</button>
				<label class="btn btn-outline btn-sm">
					Import from file…
					<input
						type="file"
						accept="application/json,.json"
						class="hidden"
						onchange={importFromFile}
					/>
				</label>
			</div>
			{#if importError}
				<p class="mt-2 text-sm text-error">{importError}</p>
			{/if}
		</section>

		<section class="card bg-base-200 p-4">
			<h2 class="mb-2 text-lg font-semibold">Reset</h2>
			<p class="mb-3 text-sm text-base-content/60">
				Generate a new key pair and register it. The current identity will be discarded.
			</p>
			<button
				class="btn btn-outline btn-sm btn-warning"
				disabled={regenerating}
				onclick={regenerate}
			>
				{regenerating ? 'Regenerating…' : 'Regenerate identity'}
			</button>
			{#if actionError}
				<p class="mt-2 text-sm text-error">{actionError}</p>
			{/if}
		</section>
	{/if}
</div>
