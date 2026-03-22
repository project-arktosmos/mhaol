<script lang="ts">
	import classNames from 'classnames';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import { modalRouterService } from 'frontend/services/modal-router.service';

	interface TabbedModalSection {
		id: string;
		label: string;
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		component: any;
	}

	let { sections, title }: { sections: TabbedModalSection[]; title?: string } = $props();

	const routerStore = modalRouterService.store;
	let activeId = $derived($routerStore.navbarModal);
	let activeSection = $derived(activeId ? sections.find((s) => s.id === activeId) : undefined);

	function handleClose() {
		modalRouterService.closeNavbar();
	}
</script>

{#if activeId && activeSection}
	<Modal open={true} maxWidth="max-w-6xl" onclose={handleClose}>
		<div class="-m-6 -mt-8 flex min-h-[70vh]">
			<!-- Desktop sidebar -->
			<ul
				class="menu hidden w-52 shrink-0 gap-1 rounded-l-2xl border-r border-base-300 bg-base-200 pt-4 lg:flex"
			>
				{#if title}
					<li class="menu-title">{title}</li>
				{/if}
				{#each sections as section}
					<li>
						<button
							class={classNames({ active: section.id === activeId })}
							onclick={() => modalRouterService.openNavbar(section.id)}
						>
							{section.label}
						</button>
					</li>
				{/each}
			</ul>

			<!-- Mobile dropdown -->
			<div class="absolute top-2 left-4 z-10 lg:hidden">
				<select
					class="select-bordered select select-sm"
					value={activeId}
					onchange={(e) => modalRouterService.openNavbar(e.currentTarget.value)}
				>
					{#each sections as section}
						<option value={section.id}>{section.label}</option>
					{/each}
				</select>
			</div>

			<!-- Content area -->
			<div class="min-w-0 flex-1 overflow-y-auto p-6 pt-12 lg:pt-6">
				<activeSection.component />
			</div>
		</div>
	</Modal>
{/if}
