<script lang="ts">
	import { profileService } from 'ui-lib/services/profile.service';
	import { identityAdapter } from 'ui-lib/adapters/classes/identity.adapter';

	const store = profileService.state;

	function handleUsernameInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		profileService.updateUsername(value);
	}

	async function handleShare() {
		await profileService.shareWithNode();
	}

	function handleRefresh() {
		profileService.refreshRemote();
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Profile</h1>
		<p class="text-sm opacity-70">Your username and wallet, stored locally.</p>
	</div>

	<div class="card bg-base-200 mb-6">
		<div class="card-body gap-4">
			<h2 class="card-title text-lg">Your Profile</h2>
			{#if $store.local.username || $store.local.wallet}
				<label class="form-control w-full">
					<div class="label"><span class="label-text">Username</span></div>
					<input
						type="text"
						class="input input-bordered w-full"
						placeholder="Enter username"
						value={$store.local.username}
						oninput={handleUsernameInput}
					/>
				</label>
				<div class="form-control w-full">
					<div class="label"><span class="label-text">Wallet</span></div>
					<code class="break-all rounded-lg bg-base-300 p-3 text-sm">
						{$store.local.wallet || 'No wallet found'}
					</code>
				</div>
				<div class="card-actions justify-end">
					<button
						class="btn btn-primary btn-sm"
						onclick={handleShare}
					>
						Share with Node
					</button>
				</div>
			{:else}
				<p class="text-sm opacity-50">
					No identity found. Create one in the node first.
				</p>
			{/if}
		</div>
	</div>

	<div class="mb-4 flex items-center justify-between">
		<div>
			<h2 class="text-xl font-bold">Profiles on this Node</h2>
			<p class="text-sm opacity-70">All profiles shared with the connected node instance.</p>
		</div>
		<button
			class="btn btn-primary btn-sm"
			onclick={handleRefresh}
			disabled={$store.loading}
		>
			{#if $store.loading}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Refresh
			{/if}
		</button>
	</div>

	{#if $store.error}
		<div class="mb-4 alert alert-error">
			<span>{$store.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => profileService.state.update((s) => ({ ...s, error: null }))}
			>
				x
			</button>
		</div>
	{/if}

	{#if $store.loading && $store.remoteProfiles.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if $store.remoteProfiles.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No profiles shared with this node yet.</p>
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each $store.remoteProfiles as profile (profile.wallet)}
				<a href="/profiles/{profile.wallet}" class="card bg-base-200 transition-colors hover:bg-base-300">
					<div class="card-body flex-row items-center gap-3 p-4">
						<div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-primary text-primary-content font-bold">
							{profile.username.charAt(0).toUpperCase()}
						</div>
						<div class="min-w-0 flex-1">
							<span class="font-semibold">{profile.username}</span>
							<code class="block break-all text-xs opacity-50">
								{identityAdapter.shortAddress(profile.wallet)}
							</code>
						</div>
						<span class="badge badge-outline badge-sm">
							{profile.favoriteCount} fav{profile.favoriteCount === 1 ? '' : 's'}
						</span>
					</div>
				</a>
			{/each}
		</div>

		<div class="mt-4 text-xs text-base-content/40">
			{$store.remoteProfiles.length} profile{$store.remoteProfiles.length === 1 ? '' : 's'} on node
		</div>
	{/if}
</div>
