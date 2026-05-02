<script lang="ts">
	import Modal from '$components/core/Modal.svelte';
	import { modalRouterService } from '$services/modal-router.service';
	interface ModalConfig {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		component: any;
		maxWidth?: string;
	}

	let { modals }: { modals: Record<string, ModalConfig> } = $props();

	const routerStore = modalRouterService.store;
	let activeId = $derived($routerStore.navbarModal);
	let config = $derived(activeId ? modals[activeId] : undefined);

	function handleClose() {
		modalRouterService.closeNavbar();
	}
</script>

{#if activeId && config}
	<Modal open={true} maxWidth={config.maxWidth ?? 'max-w-lg'} onclose={handleClose}>
		<config.component />
	</Modal>
{/if}
