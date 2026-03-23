# App: signaling

**Location:** `apps/signaling/`
**Type:** Standalone Rust binary — self-hosted signaling + TURN credential server
**Binary:** `mhaol-signaling`

## Purpose

Self-hosted replacement for the PartyKit + Metered.ca stack. Provides:
1. WebSocket signaling server (protocol-compatible with the PartyKit implementation)
2. TURN credential generation via coturn's HMAC-SHA1 shared-secret mechanism
3. CLI setup wizard for deploying to Linux servers (Ubuntu, Debian, CentOS, RHEL, Fedora)

## Source Structure

```
src/
├── main.rs              # CLI entry (clap): serve, setup, status
├── lib.rs               # Module re-exports
├── config.rs            # TOML config + env var overrides
├── server.rs            # Axum HTTP/WS server assembly + optional TLS
├── rooms.rs             # In-memory room manager (peers, relay, lifecycle)
├── ws.rs                # WebSocket handler with EIP-191 auth + passport verification
├── turn.rs              # HMAC-SHA1 TURN credential generation + REST API
├── status.rs            # Health check command
└── setup/
    ├── mod.rs            # Interactive setup wizard orchestrator
    ├── detect.rs         # OS/distro detection (apt/dnf/yum)
    ├── coturn.rs         # coturn installation and configuration
    ├── tls.rs            # certbot / Let's Encrypt setup
    └── systemd.rs        # Service file generation
```

## CLI Commands

```bash
mhaol-signaling serve [--config path]     # Run the signaling server
mhaol-signaling setup                     # Interactive setup wizard for Linux
mhaol-signaling status [--url http://...] # Health check
```

## Signaling Protocol

Wire-compatible with the PartyKit signaling server (`packages/signaling/`). Same:
- WebSocket endpoint: `/party/{room_id}`
- Auth: EIP-191 signature with `partykit-auth:{roomId}:{timestamp}` message
- Messages: connected, room-peers, peer-joined, peer-left, offer, answer, ice-candidate
- ICE servers distributed in `connected` message

## TURN Credential REST API

`GET /api/v1/turn/credentials?apiKey={key}` — returns Metered-compatible JSON format.

## Configuration

TOML config file with env var overrides:

| Env Var | Config Key | Description |
|---------|-----------|-------------|
| `SIGNALING_HOST` | `server.host` | Bind address (default: 0.0.0.0) |
| `SIGNALING_PORT` | `server.port` | Port (default: 8443) |
| `TLS_CERT` | `server.tls_cert` | TLS certificate path |
| `TLS_KEY` | `server.tls_key` | TLS private key path |
| `TURN_DOMAIN` | `turn.domain` | TURN server domain |
| `TURN_SHARED_SECRET` | `turn.shared_secret` | Shared secret (must match coturn) |
| `TURN_API_KEY` | `auth.api_keys` | API key for credential REST endpoint |

## Dependencies

- `mhaol-identity` — EIP-191 signature recovery for WebSocket auth
- `axum` + `axum-server` — HTTP/WS server with optional TLS
- `hmac` + `sha1` — TURN credential generation
- `dialoguer` — Interactive CLI prompts
