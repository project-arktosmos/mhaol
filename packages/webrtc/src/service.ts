import { writable, get } from 'svelte/store';
import { verifyPassport } from 'webrtc/verify';
import type {
	ContactHandshakeState,
	ContactHandshakeConfig,
	ContactHandshakeMessage,
	DataChannelContactEnvelope,
	PassportData,
	PendingContactRequest,
	AcceptedContact,
	Endorsement,
	WebRTCAdapter,
	ContactHandshakeCallbacks
} from 'webrtc/types';

const STORAGE_KEY = 'webrtc-contacts';

function loadPersisted(): { contacts: AcceptedContact[]; outgoingRequestAddresses: string[] } {
	if (typeof window === 'undefined') return { contacts: [], outgoingRequestAddresses: [] };
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const parsed = JSON.parse(stored);
			return {
				contacts: parsed.contacts ?? [],
				outgoingRequestAddresses: parsed.outgoingRequestAddresses ?? []
			};
		}
	} catch {
		// ignore
	}
	return { contacts: [], outgoingRequestAddresses: [] };
}

function savePersisted(contacts: AcceptedContact[], outgoingRequestAddresses: string[]): void {
	if (typeof window === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify({ contacts, outgoingRequestAddresses }));
}

const initialState: ContactHandshakeState = {
	...loadPersisted(),
	pendingRequests: [],
	peerPhases: {}
};

class ContactHandshakeService {
	public state = writable<ContactHandshakeState>(initialState);

	private localPassport: PassportData | null = null;
	private adapter: WebRTCAdapter | null = null;
	private callbacks: ContactHandshakeCallbacks | null = null;
	private peerIdToAddress: Map<string, string> = new Map();
	private pendingEndorsements: Map<string, Endorsement> = new Map();

	initialize(config: ContactHandshakeConfig): void {
		this.localPassport = config.passport;
		this.adapter = config.adapter;
		this.callbacks = config.callbacks;
	}

	handleChannelOpen(peerId: string): void {
		if (!this.adapter || !this.localPassport) return;

		const phase = this.getPeerPhase(peerId);

		// If we're in the acceptance flow, send our passport as acceptance
		if (phase === 'sending-acceptance') {
			this.sendAcceptance(peerId);
			return;
		}

		// Already waiting for acceptance — don't re-send
		if (phase === 'request-sent') {
			return;
		}

		// Check if this peer is already a known contact (by ephemeral peer ID mapping)
		const knownAddress = this.peerIdToAddress.get(peerId);
		if (knownAddress && this.isKnownContact(knownAddress)) {
			// Already a contact — skip handshake, fire connection ready
			const contact = get(this.state).contacts.find(
				(c) => c.address.toLowerCase() === knownAddress.toLowerCase()
			);
			if (contact) this.callbacks?.onConnectionReady(peerId, contact);
			return;
		}

		// Check if we are the initiator (offering) or receiver (answering)
		const status = this.adapter.getPeerConnectionStatus(peerId);

		if (status === 'offering' || status === 'connected') {
			// We initiated — send contact request
			this.setPeerPhase(peerId, 'sending-request');
			const envelope: DataChannelContactEnvelope = {
				channel: 'contact',
				payload: { type: 'contact-request', passport: this.localPassport }
			};
			this.adapter.sendToPeer(peerId, envelope);
			this.setPeerPhase(peerId, 'request-sent');

			// Track that we sent a request (by our own address, to match when they accept)
			const localPayload = JSON.parse(this.localPassport.raw);
			this.state.update((s) => {
				const updated = {
					...s,
					outgoingRequestAddresses: [...s.outgoingRequestAddresses, localPayload.address]
				};
				savePersisted(updated.contacts, updated.outgoingRequestAddresses);
				return updated;
			});
		}
		// If answering, we wait for the contact-request message
	}

	async handleMessage(peerId: string, msg: ContactHandshakeMessage): Promise<void> {
		switch (msg.type) {
			case 'contact-request':
				await this.handleContactRequest(peerId, msg.passport);
				break;
			case 'contact-accept':
				await this.handleContactAccept(peerId, msg.passport, msg.endorsement);
				break;
		}
	}

	acceptRequest(address: string, endorsement?: Endorsement): void {
		if (!this.adapter) return;

		const s = get(this.state);
		const request = s.pendingRequests.find(
			(r) => r.address.toLowerCase() === address.toLowerCase()
		);
		if (!request) return;

		// Store endorsement for inclusion in the acceptance message
		if (endorsement) {
			this.pendingEndorsements.set(request.peerId, endorsement);
		}

		// Add to contacts
		const contact: AcceptedContact = {
			name: request.name,
			address: request.address,
			passport: request.passport,
			acceptedAt: new Date().toISOString(),
			endorsement,
			username: request.username,
			profilePictureUrl: request.profilePictureUrl
		};

		this.state.update((s) => {
			const updated = {
				...s,
				contacts: [...s.contacts, contact],
				pendingRequests: s.pendingRequests.filter(
					(r) => r.address.toLowerCase() !== address.toLowerCase()
				),
				peerPhases: { ...s.peerPhases, [request.peerId]: 'sending-acceptance' as const }
			};
			savePersisted(updated.contacts, updated.outgoingRequestAddresses);
			return updated;
		});

		// Connect back to send our passport
		this.adapter.connectToPeer(request.peerId);
	}

	rejectRequest(address: string): void {
		this.state.update((s) => {
			const updated = {
				...s,
				pendingRequests: s.pendingRequests.filter(
					(r) => r.address.toLowerCase() !== address.toLowerCase()
				)
			};
			return updated;
		});
	}

	isKnownContact(address: string): boolean {
		return get(this.state).contacts.some(
			(c) => c.address.toLowerCase() === address.toLowerCase()
		);
	}

	getIdentityAddress(peerId: string): string | null {
		return this.peerIdToAddress.get(peerId) ?? null;
	}

	removeContact(address: string): void {
		this.state.update((s) => {
			const updated = {
				...s,
				contacts: s.contacts.filter(
					(c) => c.address.toLowerCase() !== address.toLowerCase()
				)
			};
			savePersisted(updated.contacts, updated.outgoingRequestAddresses);
			return updated;
		});
	}

	destroy(): void {
		this.localPassport = null;
		this.adapter = null;
		this.callbacks = null;
		this.peerIdToAddress.clear();
		this.pendingEndorsements.clear();
	}

	// ===== Private =====

	private async handleContactRequest(
		peerId: string,
		passport: PassportData
	): Promise<void> {
		try {
			const payload = await verifyPassport(passport);

			// If already a contact, send acceptance back and fire connection ready
			if (this.isKnownContact(payload.address)) {
				this.peerIdToAddress.set(peerId, payload.address);
				this.sendAcceptance(peerId);
				return;
			}

			// Store the peerId → address mapping
			this.peerIdToAddress.set(peerId, payload.address);

			// Close the connection per protocol
			this.adapter?.disconnectPeer(peerId);

			const request: PendingContactRequest = {
				peerId,
				passport,
				name: payload.name,
				address: payload.address,
				username: payload.username,
				profilePictureUrl: payload.profilePictureUrl,
				receivedAt: new Date().toISOString()
			};

			this.state.update((s) => ({
				...s,
				pendingRequests: [...s.pendingRequests, request],
				peerPhases: { ...s.peerPhases, [peerId]: 'request-received' }
			}));

			this.callbacks?.onRequestReceived(request);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Invalid passport';
			this.callbacks?.onError(`Invalid contact request: ${message}`);
		}
	}

	private async handleContactAccept(
		peerId: string,
		passport: PassportData,
		endorsement?: Endorsement
	): Promise<void> {
		try {
			const payload = await verifyPassport(passport);

			this.peerIdToAddress.set(peerId, payload.address);

			const contact: AcceptedContact = {
				name: payload.name,
				address: payload.address,
				passport,
				acceptedAt: new Date().toISOString(),
				endorsement,
				username: payload.username,
				profilePictureUrl: payload.profilePictureUrl
			};

			const alreadyKnown = this.isKnownContact(payload.address);

			this.state.update((s) => {
				const updated = {
					...s,
					contacts: alreadyKnown ? s.contacts : [...s.contacts, contact],
					outgoingRequestAddresses: s.outgoingRequestAddresses.filter(
						(a) => a.toLowerCase() !== payload.address.toLowerCase()
					),
					peerPhases: { ...s.peerPhases, [peerId]: 'accepted' as const }
				};
				savePersisted(updated.contacts, updated.outgoingRequestAddresses);
				return updated;
			});

			if (!alreadyKnown) {
				this.callbacks?.onRequestAccepted(contact);
			}
			this.callbacks?.onConnectionReady(peerId, contact);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Invalid passport';
			this.callbacks?.onError(`Invalid contact acceptance: ${message}`);
		}
	}

	private sendAcceptance(peerId: string): void {
		if (!this.localPassport || !this.adapter) return;

		const endorsement = this.pendingEndorsements.get(peerId);
		this.pendingEndorsements.delete(peerId);

		const envelope: DataChannelContactEnvelope = {
			channel: 'contact',
			payload: { type: 'contact-accept', passport: this.localPassport, endorsement }
		};
		this.adapter.sendToPeer(peerId, envelope);
		this.setPeerPhase(peerId, 'accepted');

		// Fire connection ready — the channel is open and contact is accepted
		const address = this.peerIdToAddress.get(peerId);
		if (address) {
			const contact = get(this.state).contacts.find(
				(c) => c.address.toLowerCase() === address.toLowerCase()
			);
			if (contact) this.callbacks?.onConnectionReady(peerId, contact);
		}
	}

	private getPeerPhase(peerId: string): string | undefined {
		return get(this.state).peerPhases[peerId];
	}

	private setPeerPhase(
		peerId: string,
		phase: ContactHandshakeState['peerPhases'][string]
	): void {
		this.state.update((s) => ({
			...s,
			peerPhases: { ...s.peerPhases, [peerId]: phase }
		}));
	}
}

export const contactHandshakeService = new ContactHandshakeService();
