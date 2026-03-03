<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';

	interface Props {
		open?: boolean;
		maxWidth?: string;
		zIndex?: number;
		onclose?: () => void;
		children?: Snippet;
	}

	let { open = false, maxWidth = 'max-w-lg', zIndex = 50, onclose, children }: Props = $props();

	let boxClasses = $derived(classNames('modal-box max-h-[90vh] overflow-y-auto', maxWidth));

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') onclose?.();
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="modal-open modal"
		style:z-index={zIndex}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class={boxClasses}>
			<button
				class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm"
				onclick={() => onclose?.()}
			>
				&times;
			</button>
			{#if children}
				{@render children()}
			{/if}
		</div>
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<div class="modal-backdrop" onclick={() => onclose?.()}></div>
	</div>
{/if}
