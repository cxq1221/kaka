# ADR 0114: Explicit LanChat screenshot sharing

Status: Accepted

## Context

Captures normally remain in Kaka's local library and clipboard. A useful phone
handoff already exists in the local LanChat product: its `/api/files` endpoint
creates a file message that appears in the room history and can be opened from
the phone. Adding this handoff changes Kaka's previous no-network product
contract, so the trigger, destination, identity, and failure behavior must be
explicit.

## Decision

- Sharing is an explicit action from the screenshot toolbar or from a saved
  image's completion card. Kaka never uploads immediately after capture, in the
  background, or as a fallback.
- The React renderer requests a share through Tauri IPC. Rust owns address
  validation, room discovery, HTTP, the persistent LanChat identity, and
  reading the managed library file. No local filesystem path or HTTP capability
  is exposed to the renderer.
- Automatic discovery probes loopback ports `81` and `8787`. A user may save an
  optional `http://` address for a custom port or a trusted-LAN host. Only
  loopback, private IPv4, link-local IPv4, unique-local IPv6, link-local IPv6,
  and `localhost` are accepted; credentials, paths, queries, fragments, HTTPS,
  and public hosts are rejected.
- A candidate is usable only when `/api/health` returns product `lan-chat`,
  protocol `1`, a valid room UUID, a valid instance UUID, and the port used by
  the candidate origin. Another service on a candidate port is ignored.
- Sharing sends the selected library file as the request body to `/api/files`
  with `Content-Type: application/octet-stream` and a UTF-8 percent-encoded
  `X-File-Name`. The generated display name uses `kaka-` plus the capture
  timestamp and a short asset id.
- The pre-completion Share action first completes the normal local save and
  clipboard path, then starts the network request after the overlay is
  dismissed. It never uploads when local persistence fails. The completion card
  exposes sharing only after clipboard and library persistence succeed. Unlike
  passive feedback, this interactive card remains visible in screen sharing and
  remote-desktop capture so the action can be discovered; capture startup still
  hides it before freezing the next frame.
- The `lan_identity` value returned by LanChat is stored in Kaka's application
  configuration directory and reused only for the same room UUID. It is a
  random room identity, not an account or secret; the request itself remains
  unauthenticated plain HTTP.
- Sharing is best-effort additive behavior. Clipboard and library persistence
  complete before the action is available. A missing server, incompatible
  room, timeout, or upload error only produces localized feedback and never
  deletes or changes the capture.

## Security and privacy boundary

LanChat transmits and stores the selected flattened image on the room host
without transport encryption. This feature therefore supports only a local or
user-specified private-network room that the user trusts. It does not share the
editable clean source, annotation document, library index, OCR history, or
other captures. The LanChat host's own retention and download behavior remain
outside Kaka.

## Verification boundary

Unit tests cover address policy, health validation, UTF-8 header encoding,
identity extraction, a successful upload, and reuse of the saved identity on a
second upload. The normal Rust, TypeScript/Vite, and repository checks remain
required. A live LanChat run verifies the actual message history; physical
phone gallery saving remains a LanChat behavior rather than a Kaka claim.
