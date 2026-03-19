<script lang="ts">
	import { onMount } from 'svelte';
	import HubDashboard from 'ui-lib/components/hub/HubDashboard.svelte';
	import { hubService } from 'frontend/services/hub.service';

	const state = hubService.state;

	onMount(() => {
		hubService.initialize();
		return () => hubService.destroy();
	});
</script>

<div class="mx-auto max-w-4xl p-6">
	<HubDashboard
		apps={$state.apps}
		loading={$state.loading}
		error={$state.error}
		onbuild={(name) => hubService.buildApp(name)}
		onstart={(name) => hubService.startApp(name)}
		onstop={(name) => hubService.stopApp(name)}
		ondismiss={(name) => hubService.dismissApp(name)}
	/>
</div>
