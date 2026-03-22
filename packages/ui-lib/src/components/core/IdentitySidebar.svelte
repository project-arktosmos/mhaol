<script lang="ts">
	import classNames from 'classnames';
	import { sidebarService } from 'ui-lib/services/sidebar.service';
	import DownloadsSummary from 'ui-lib/components/downloads/DownloadsSummary.svelte';
	import type { SidebarWidthMode } from 'ui-lib/types/sidebar.type';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	const sidebarSettings = sidebarService.store;

	const widthClasses: Record<SidebarWidthMode, string> = {
		wide: 'w-[80vw]',
		default: 'w-128',
		narrow: 'w-85'
	};

	let wrapperClasses = $derived(
		classNames(
			'hidden lg:flex flex-col bg-base-200 border-l border-base-300 p-4 overflow-y-auto',
			widthClasses[$sidebarSettings.widthMode],
			classes
		)
	);

	const widthModes: { mode: SidebarWidthMode; label: string }[] = [
		{ mode: 'wide', label: 'Wide' },
		{ mode: 'default', label: 'Default' },
		{ mode: 'narrow', label: 'Narrow' }
	];
</script>

<aside class={wrapperClasses}>
	<div class="mb-4 flex justify-center">
		<div class="join">
			{#each widthModes as { mode, label }}
				<button
					class={classNames('btn join-item btn-xs', {
						'btn-active': $sidebarSettings.widthMode === mode
					})}
					onclick={() => sidebarService.setWidthMode(mode)}
				>
					{label}
				</button>
			{/each}
		</div>
	</div>

	<DownloadsSummary />
</aside>
