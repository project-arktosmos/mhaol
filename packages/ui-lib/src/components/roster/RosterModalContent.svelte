<script lang="ts">
	import classNames from 'classnames';
	import { rosterService } from 'ui-lib/services/roster.service';
	import { identityAdapter } from 'ui-lib/adapters/classes/identity.adapter';
	import type { RosterPeerStatus } from 'ui-lib/types/roster.type';

	const rosterStore = rosterService.state;

	function handleRefresh() {
		rosterService.refresh();
	}

	function statusDotClass(status: RosterPeerStatus): string {
		const map: Record<RosterPeerStatus, string> = {
			online: 'bg-success',
			checking: 'bg-warning animate-pulse',
			offline: 'bg-base-content/20'
		};
		return map[status];
	}

	function statusLabel(status: RosterPeerStatus): string {
		const labels: Record<RosterPeerStatus, string> = {
			online: 'Online',
			checking: 'Checking...',
			offline: 'Offline'
		};
		return labels[status];
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Roster</h3>
	<p class="text-sm text-base-content/60">
		Known identities and their online status via signaling.
	</p>
</div>

{#if $rosterStore.error}
	<div class="mt-4 alert alert-error">
		<span>{$rosterStore.error}</span>
		<button
			class="btn btn-ghost btn-sm"
			onclick={() => rosterService.state.update((s) => ({ ...s, error: null }))}
		>
			x
		</button>
	</div>
{/if}

<!-- Controls -->
<div class="mt-4 flex items-end gap-2">
	<div class="flex-1"></div>
	<button class="btn btn-sm btn-primary" onclick={handleRefresh} disabled={$rosterStore.loading}>
		{#if $rosterStore.loading}
			<span class="loading loading-xs loading-spinner"></span>
		{:else}
			Refresh
		{/if}
	</button>
</div>

<!-- Roster list -->
{#if $rosterStore.loading && $rosterStore.entries.length === 0}
	<div class="flex justify-center py-12">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if $rosterStore.entries.length === 0}
	<div class="mt-4 rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No contacts in roster yet.</p>
		<p class="mt-1 text-sm opacity-40">
			Contacts are added when peers exchange passports via signaling.
		</p>
	</div>
{:else}
	<div class="mt-4 flex flex-col gap-2">
		{#each $rosterStore.entries as entry (entry.address)}
			<div class="card bg-base-200">
				<div class="card-body flex-row items-center gap-3 p-3">
					<span class={classNames('h-3 w-3 shrink-0 rounded-full', statusDotClass(entry.status))}
					></span>
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="font-mono text-sm font-semibold">{entry.name}</span>
							{#if entry.instanceType}
								<span class="badge badge-outline badge-xs">{entry.instanceType}</span>
							{/if}
						</div>
						<code class="block text-xs break-all opacity-50">
							{identityAdapter.shortAddress(entry.address)}
						</code>
					</div>
					<span
						class={classNames('badge badge-sm', {
							'badge-success': entry.status === 'online',
							'badge-warning': entry.status === 'checking',
							'badge-ghost': entry.status === 'offline'
						})}
					>
						{statusLabel(entry.status)}
					</span>
				</div>
			</div>
		{/each}
	</div>

	<!-- Summary -->
	<div class="mt-3 text-xs text-base-content/40">
		{$rosterStore.entries.filter((e) => e.status === 'online').length} of {$rosterStore.entries
			.length} online
	</div>
{/if}
