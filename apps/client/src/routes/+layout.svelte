<script lang="ts">
	import '../css/app.css';
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import RosterModalContent from 'ui-lib/components/roster/RosterModalContent.svelte';
	import { themeService } from 'frontend/services/theme.service';
	import { rosterService } from 'frontend/services/roster.service';

	let { children } = $props();

	const rosterStore = rosterService.state;

	const navItems = [{ id: 'roster', label: 'Roster', classes: 'btn-primary' }];

	const modals: Record<string, { component: typeof RosterModalContent; maxWidth?: string }> = {
		roster: { component: RosterModalContent, maxWidth: 'max-w-2xl' }
	};

	onMount(() => {
		themeService.initialize();
		rosterService.initialize();
	});
</script>

<Navbar brand={{ label: 'Mhaol Client' }} items={navItems}>
	{#snippet end()}
		<div
			class="flex items-center gap-2"
			title={$rosterStore.signalingServerUrl || 'Signaling server not available'}
		>
			<span
				class={classNames('h-2.5 w-2.5 rounded-full', {
					'bg-success': $rosterStore.signalingServerUrl,
					'bg-error': !$rosterStore.signalingServerUrl
				})}
			></span>
			<span class="hidden text-xs text-base-content/60 sm:inline">
				{$rosterStore.signalingServerUrl ? 'Signaling' : 'Offline'}
			</span>
		</div>
		<ThemeToggle />
	{/snippet}
</Navbar>
<main class="p-4">
	{@render children()}
</main>
<ModalOutlet {modals} />
