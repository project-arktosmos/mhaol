<script lang="ts">
	import '../css/app.css';
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import RosterModalContent from 'ui-lib/components/roster/RosterModalContent.svelte';
	import { themeService } from 'frontend/services/theme.service';
	import { rosterService } from 'frontend/services/roster.service';
	import { identityService } from 'frontend/services/identity.service';
	import { toastService } from 'frontend/services/toast.service';
	import { signalingChatService } from 'frontend/services/signaling-chat.service';
	import { signalingAdapter } from 'frontend/adapters/classes/signaling.adapter';
	import { contactHandshakeService } from 'webrtc/service';
	import type { PassportData } from 'webrtc/types';
	import type { ContactHandshakeMessage } from 'webrtc/types';

	let { children } = $props();

	const rosterStore = rosterService.state;
	const chatStore = signalingChatService.state;

	const navItems = [{ id: 'roster', label: 'Roster', classes: 'btn-primary' }];

	const modals: Record<string, { component: typeof RosterModalContent; maxWidth?: string }> = {
		roster: { component: RosterModalContent, maxWidth: 'max-w-2xl' }
	};

	let prevPhase: string | null = null;

	onMount(() => {
		themeService.initialize();
		rosterService.initialize();

		// Initialize identity and contact handshake asynchronously
		identityService.initialize().then(() => {
			const identities = get(identityService.state).identities;
			if (identities.length > 0 && identities[0].passport) {
				const passport: PassportData = JSON.parse(identities[0].passport);
				contactHandshakeService.initialize({
					passport,
					adapter: {
						sendToPeer: (peerId, envelope) => signalingChatService.sendToPeer(peerId, envelope),
						disconnectPeer: (peerId) => signalingChatService.disconnectPeer(peerId),
						connectToPeer: (peerId) => signalingChatService.connectToPeer(peerId),
						getPeerConnectionStatus: (peerId) =>
							signalingChatService.getPeerConnectionStatus(peerId)
					},
					callbacks: {
						onRequestReceived: (request) => {
							toastService.addWithActions(
								`Contact request from ${request.name} (${signalingAdapter.shortAddress(request.address)})`,
								[
									{
										label: 'Accept',
										onclick: () => contactHandshakeService.acceptRequest(request.address)
									},
									{
										label: 'Reject',
										onclick: () => contactHandshakeService.rejectRequest(request.address)
									}
								],
								'info'
							);
						},
						onRequestAccepted: (contact) => {
							toastService.success(`${contact.name} accepted your contact request`);
						},
						onError: (message) => {
							toastService.error(message);
						}
					}
				});

				signalingChatService.addPeerChannelOpenListener((peerId) =>
					contactHandshakeService.handleChannelOpen(peerId)
				);
				signalingChatService.onContactMessage = (peerId, msg) =>
					contactHandshakeService.handleMessage(peerId, msg as ContactHandshakeMessage);
			}
		});

		const unsubChat = chatStore.subscribe((s) => {
			if (prevPhase === null) {
				prevPhase = s.phase;
				return;
			}
			if (s.phase === prevPhase) return;

			switch (s.phase) {
				case 'connecting':
					toastService.info('Connecting to signaling server...');
					break;
				case 'connected':
					toastService.success('Connected to signaling server');
					break;
				case 'error':
					toastService.error(s.error || 'Connection error');
					break;
				case 'disconnected':
					if (prevPhase === 'connected') {
						toastService.warning('Disconnected from signaling server');
					}
					break;
			}
			prevPhase = s.phase;
		});

		return () => {
			unsubChat();
			contactHandshakeService.destroy();
		};
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
<ToastOutlet />
