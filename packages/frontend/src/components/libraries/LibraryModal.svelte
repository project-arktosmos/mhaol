<script lang="ts">
	import classNames from 'classnames';
	import { libraryService } from '$services/library.service';
	import { libraryModalService } from '$services/library-modal.service';
	import LibraryAddForm from './LibraryAddForm.svelte';
	import LibraryList from './LibraryList.svelte';

	const modalOpen = libraryModalService.store;

	let activeTab: 'add' | 'libraries' = $state('libraries');
	let initialized = $state(false);

	$effect(() => {
		if ($modalOpen && !initialized) {
			initialized = true;
			libraryService.initialize();
		}
		if (!$modalOpen && initialized) {
			initialized = false;
			activeTab = 'libraries';
		}
	});

	function handleClose() {
		libraryModalService.close();
	}
</script>

{#if $modalOpen}
	<div class="modal modal-open">
		<div class="modal-box max-h-[90vh] max-w-5xl overflow-y-auto">
			<button
				class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
				onclick={handleClose}
			>
				&times;
			</button>

			<div class="flex items-center justify-between pr-8">
				<div>
					<h3 class="text-lg font-bold">Libraries</h3>
					<p class="text-sm text-base-content/60">
						Manage media library locations on your server
					</p>
				</div>
			</div>

			<div class="mt-4 flex gap-2">
				<div class="join">
					<button
						class={classNames('join-item btn btn-sm', {
							'btn-active': activeTab === 'libraries'
						})}
						onclick={() => (activeTab = 'libraries')}
					>
						Libraries
					</button>
					<button
						class={classNames('join-item btn btn-sm', {
							'btn-active': activeTab === 'add'
						})}
						onclick={() => (activeTab = 'add')}
					>
						Add Library
					</button>
				</div>
			</div>

			<div class="mt-4">
				{#if activeTab === 'add'}
					<LibraryAddForm />
				{:else}
					<LibraryList />
				{/if}
			</div>
		</div>
		<div class="modal-backdrop" onclick={handleClose}></div>
	</div>
{/if}
