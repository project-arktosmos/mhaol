<script lang="ts">
	import classNames from 'classnames';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
	import { peerLibraryAdapter } from 'ui-lib/adapters/classes/peer-library.adapter';
	import { peerLibraryService } from 'ui-lib/services/peer-library.service';
	import PeerLibraryFileList from './PeerLibraryFileList.svelte';
	import type { PeerLibraryData } from 'ui-lib/types/peer-library.type';

	let {
		peerId,
		peerData
	}: {
		peerId: string;
		peerData: PeerLibraryData;
	} = $props();

	let expandedLibraryId: string | null = $state(null);

	function toggleLibrary(libraryId: string) {
		if (expandedLibraryId === libraryId) {
			expandedLibraryId = null;
			return;
		}

		expandedLibraryId = libraryId;

		if (!peerData.files[libraryId] && !peerData.filesLoading[libraryId]) {
			peerLibraryService.requestFiles(peerId, libraryId);
		}
	}

	function libraryTypeBadgeClass(type: string): string {
		if (type === 'movies') return 'badge-secondary';
		if (type === 'tv') return 'badge-accent';
		return 'badge-neutral';
	}
</script>

<div class="rounded-lg bg-base-200 p-3">
	<div class="mb-2 flex items-center gap-2">
		<span class="font-mono text-sm font-medium">{signalingAdapter.shortAddress(peerId)}</span>
		<span class="badge badge-sm badge-success">connected</span>
	</div>

	{#if peerData.libraries.length === 0}
		<p class="text-sm opacity-50">No libraries shared.</p>
	{:else}
		<div class="flex flex-col gap-1">
			{#each peerData.libraries as library (library.id)}
				<div class="rounded-md bg-base-300">
					<button
						class="flex w-full items-center justify-between px-3 py-2 text-left text-sm transition-colors hover:bg-base-100/50"
						onclick={() => toggleLibrary(library.id)}
					>
						<div class="flex items-center gap-2">
							<span
								class={classNames(
									'text-xs transition-transform',
									expandedLibraryId === library.id && 'rotate-90'
								)}
							>
								&#9654;
							</span>
							<span class="font-medium">{library.name}</span>
							<span
								class={classNames('badge badge-sm', libraryTypeBadgeClass(library.libraryType))}
							>
								{peerLibraryAdapter.libraryTypeLabel(library.libraryType)}
							</span>
						</div>
						<span class="text-xs opacity-60">{library.fileCount} files</span>
					</button>

					{#if expandedLibraryId === library.id}
						<div class="border-t border-base-100 px-3 py-2">
							<PeerLibraryFileList
								files={peerData.files[library.id] ?? []}
								loading={peerData.filesLoading[library.id] ?? false}
							/>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
