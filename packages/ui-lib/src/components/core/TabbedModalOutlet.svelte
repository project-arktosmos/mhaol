<script lang="ts">
	import classNames from 'classnames';
	import { modalRouterService } from 'ui-lib/services/modal-router.service';

	interface TabbedModalSection {
		id: string;
		label: string;
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		component: any;
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		props?: Record<string, any>;
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
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="modal-open modal"
		style:z-index={50}
		onkeydown={(e) => e.key === 'Escape' && handleClose()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="modal-box flex h-[90vh] max-w-6xl flex-col overflow-hidden p-0">
			{#if title}
				<div
					class="flex shrink-0 items-center justify-between border-b border-base-300 px-6 py-4"
				>
					<h3 class="text-lg font-semibold">{title}</h3>
					<button class="btn btn-circle btn-ghost btn-sm" onclick={handleClose}>
						&times;
					</button>
				</div>
			{:else}
				<button
					class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm"
					onclick={handleClose}
				>
					&times;
				</button>
			{/if}

			<div class="flex min-h-0 flex-1">
				<!-- Desktop sidebar -->
				<ul
					class={classNames(
						'menu hidden w-52 shrink-0 gap-1 border-r border-base-300 bg-base-200 pt-4 lg:flex',
						{ 'rounded-bl-2xl': true, 'rounded-tl-2xl': !title }
					)}
				>
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
				<div class="absolute left-4 z-10 lg:hidden" class:top-14={!!title} class:top-2={!title}>
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
					<activeSection.component {...(activeSection.props ?? {})} />
				</div>
			</div>
		</div>
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<div class="modal-backdrop" onclick={handleClose}></div>
	</div>
{/if}
