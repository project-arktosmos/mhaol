<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import ToastOutlet from 'ui-lib/components/core/ToastOutlet.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import SignalingStatusBadge from 'ui-lib/components/signaling/SignalingStatusBadge.svelte';
	import RosterModalContent from 'ui-lib/components/roster/RosterModalContent.svelte';
	import { themeService } from 'ui-lib/services/theme.service';
	import { rosterService } from 'ui-lib/services/roster.service';
	import { clientIdentityService } from 'ui-lib/services/client-identity.service';
	import { toastService } from 'ui-lib/services/toast.service';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
	import { contactHandshakeService } from 'webrtc/service';
	import { serverCatalogService } from 'ui-lib/services/server-catalog.service';
	import { p2pStreamService } from 'ui-lib/services/p2p-stream.service';
	import type { ContactHandshakeMessage, Endorsement } from 'webrtc/types';
	import { getAddress } from 'viem';

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
		rosterService.initialize('local');
		serverCatalogService.initialize();
		p2pStreamService.initializeLocal();

		// Initialize client-side identity and contact handshake
		clientIdentityService.initialize(DEFAULT_SIGNALING_URL).then(() => {
			const { identity } = get(clientIdentityService.state);
			if (identity) {
				contactHandshakeService.initialize({
					passport: identity.passport,
					adapter: {
						sendToPeer: (peerId, envelope) => signalingChatService.sendToPeer(peerId, envelope),
						disconnectPeer: (peerId) => signalingChatService.disconnectPeer(peerId),
						connectToPeer: (peerId) => signalingChatService.connectToPeer(peerId),
						getPeerConnectionStatus: (peerId) =>
							signalingChatService.getPeerConnectionStatus(peerId)
					},
					callbacks: {
						onRequestReceived: (request) => {
							const requestPayload = JSON.parse(request.passport.raw);
							toastService.addWithActions(
								`Contact request from ${request.name} (${signalingAdapter.shortAddress(request.address)})`,
								[
									{
										label: 'Accept',
										onclick: () => {
											contactHandshakeService.acceptRequest(request.address);
											rosterService.addEntry({
												name: request.name,
												address: request.address,
												passport: JSON.stringify(request.passport),
												instanceType: requestPayload.instanceType
											});
										}
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
							const acceptPayload = JSON.parse(contact.passport.raw);
							rosterService.addEntry({
								name: contact.name,
								address: contact.address,
								passport: JSON.stringify(contact.passport),
								instanceType: acceptPayload.instanceType,
								endorsement: contact.endorsement ? JSON.stringify(contact.endorsement) : undefined
							});

							// Auto-join server's personal room if endorsed
							if (contact.endorsement && acceptPayload.instanceType === 'server') {
								const serverRoom = getAddress(contact.address as `0x${string}`);
								const { signalingServerUrl } = get(rosterService.state);
								signalingChatService.connectToRoom(
									signalingServerUrl,
									serverRoom,
									identity.passport,
									(m) => clientIdentityService.signMessage(m),
									contact.endorsement
								);
							}
						},
						onConnectionReady: () => {
							// Server will send catalog; client receives via serverCatalogService
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

				// Connect to handshakes room
				const { signalingServerUrl } = get(rosterService.state);
				signalingChatService.connectToRoom(
					signalingServerUrl,
					'handshakes',
					identity.passport,
					(m) => clientIdentityService.signMessage(m)
				);

				// Reconnect to endorsed server rooms from stored roster entries
				const entries = get(rosterService.state).entries;
				for (const entry of entries) {
					if (entry.endorsement && entry.instanceType === 'server') {
						try {
							const endorsement: Endorsement = JSON.parse(entry.endorsement);
							const serverRoom = getAddress(entry.address as `0x${string}`);
							signalingChatService.connectToRoom(
								signalingServerUrl,
								serverRoom,
								identity.passport,
								(m) => clientIdentityService.signMessage(m),
								endorsement
							);
						} catch {
							// Skip invalid endorsements
						}
					}
				}
			}
		});

		const unsubChat = chatStore.subscribe((s) => {
			const handshakesRoom = s.rooms['handshakes'];
			const phase = handshakesRoom?.phase ?? 'disconnected';

			if (prevPhase === null) {
				prevPhase = phase;
				return;
			}
			if (phase === prevPhase) return;

			switch (phase) {
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
			prevPhase = phase;
		});

		return () => {
			unsubChat();
			rosterService.destroy();
			contactHandshakeService.destroy();
			serverCatalogService.destroy();
		};
	});
</script>

<Navbar brand={{ label: 'Mhaol Client' }} items={navItems}>
	{#snippet end()}
		<SignalingStatusBadge />
		<ThemeToggle />
	{/snippet}
</Navbar>
<main class="p-4">
	{@render children()}
</main>
<ModalOutlet {modals} />
<ToastOutlet />
