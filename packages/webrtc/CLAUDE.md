# Package: webrtc

**Location:** `packages/webrtc/`
**Role:** WebRTC contact handshake layer — passport exchange and verification
**Framework:** TypeScript + Svelte stores + viem

This package implements a contact-request handshake protocol on top of WebRTC data channels. Before peers can freely communicate, they must exchange cryptographically signed passports (EIP-191) and the receiver must explicitly accept the contact request.

## Source Structure

```
src/
├── index.ts     # Re-exports
├── types.ts     # Passport, handshake message, state, adapter types
├── verify.ts    # EIP-191 passport verification via viem
└── service.ts   # ContactHandshakeService (state machine + Svelte store)
```

## Passport Payload

Each passport signs a JSON payload with fields: `name`, `address`, `instanceType` (`"server"` or `"client"`), and `signalingUrl` (the signaling server URL the peer connects from).

## Protocol Flow

1. Initiator connects to peer via signaling → sends passport via `{ channel: 'contact' }` data channel
2. Receiver verifies passport signature, closes connection, shows accept/reject prompt
3. If accepted: receiver connects back, sends their passport; both sides register each other on their local roster
4. Both peers are now contacts and can freely connect

## Import Conventions

Within this package — use `webrtc/...` paths:

```typescript
import { verifyPassport } from 'webrtc/verify';
import type { PassportData } from 'webrtc/types';
```

## Dependencies

- `viem` — EIP-191 signature recovery (`recoverMessageAddress`)
- `svelte` (peer) — Writable stores for reactive state

## Integration

This package has **no dependency on the cloud WebUI**. Apps wire it via dependency injection:

```typescript
contactHandshakeService.initialize({
  passport: localPassport,
  adapter: { sendToPeer, disconnectPeer, connectToPeer, getPeerConnectionStatus },
  callbacks: { onRequestReceived, onRequestAccepted, onError }
});
```

Accepted contacts are persisted in localStorage under the key `webrtc-contacts`.
