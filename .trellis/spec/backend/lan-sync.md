# LAN Clipboard Sync (`src-tauri/src/sync/`)

Contract notes for the cross-device sync module. Read before changing pairing,
the wire protocol, or what gets sent to peers.

## Module map

| File | Owns |
| --- | --- |
| `identity.rs` | Local `device_id`, paired-device table (`<app_data>/state/sync-devices.json`, atomic write, long-term key in hex) |
| `pairing.rs` | 6-digit code lifecycle + SPAKE2 handshake (`run_handshake`) |
| `session.rs` | Mutual HMAC challenge/response for already-paired devices, derives session keys |
| `protocol.rs` | `Message` enum, length-prefixed frames, ChaCha20-Poly1305 `SecureReader/Writer` |
| `discovery.rs` | mDNS `_ecopaste-sync._tcp.local.` register/browse (TXT: `id`, `name`, `pair`, `v`) |
| `manager.rs` | `SyncState`: listener, supervisor, connections, broadcast, pairing entry points |
| `apply.rs` | Receive side for text/image: build item -> write system clipboard -> `persist_and_notify` |
| `files.rs` / `remote_files/` | Send-side manifest + ranged reads; receive-side virtual files (Windows OLE) |

## Security invariants

- The 6-digit code is only a SPAKE2 password; it never goes on the wire and is never
  used directly as a key. One code allows at most `MAX_ATTEMPTS` (3) handshakes, lives
  `CODE_TTL` (5 min), and is cleared on success. `begin_attempt` reserves an attempt
  *before* the handshake so concurrent guesses cannot exceed the cap.
- Long-term key = HKDF of the SPAKE2 secret salted with the sorted device ids. Session
  keys are derived per connection from fresh nonces; each direction has its own key and
  a counter nonce (replayed/reordered frames fail authentication).
- Never serialize `PairedDevice.key` to the frontend: status DTOs (`PeerStatus`) omit it.
- Frames are capped at `MAX_FRAME` (16 MiB); bulk data is chunked at `CHUNK_SIZE`.

## What is (not) sent

- Skipped: `is_sensitive` items, anything the clipboard watcher already dropped
  (excluded apps, disabled capture kinds), kinds disabled in `settings.sync`.
- Received items go through `WritebackGuard` before the clipboard write, so the local
  watcher ignores them: no re-ingest and no re-broadcast (star topology, no relay).
- Files are never pre-transferred. The sender registers a `LocalOffer`
  (max 3, 30 min TTL); the receiver places a virtual-file `IDataObject` and pulls ranges
  with `FetchFile` only when an app reads the stream.

## Gotchas

- Windows clipboard open can be refused while another app holds it: receive-side writes
  retry with backoff (`write_item_with_retry`). Do not hold `ClipboardContext` across `.await`.
- OLE clipboard objects must live on an STA thread that pumps messages; `remote_files/windows.rs`
  owns a dedicated thread per offer and exits when `OleIsCurrentClipboard` reports it was replaced.
  The stream `Read` calls `block_on` -> never invoke it from a tokio worker thread.
- macOS virtual files (`NSFilePromiseProvider`) are **not implemented** (TODO in
  `remote_files/mod.rs`); text and image sync work on both platforms.
- `windows` crate needs the `implement` feature for `#[implement(IDataObject/IStream)]`.
- Only the peer with the smaller `device_id` dials; `supervise_loop` retries every 5 s.

## Testing

- Unit tests live next to each module (crypto, pairing, framing, offers).
- `e2e_tests.rs` runs two `SyncState`s over real mDNS + TCP inside one test process. The
  system clipboard write and OLE placement are replaced by `apply::TEST_SINK` /
  `remote_files::CAPTURED`, so no desktop clipboard permission is needed. Run:
  `cargo test --lib sync:: -- --include-ignored --test-threads=1`.
- On the GNU toolchain the test exe needs a comctl32 v6 manifest (embed one, see
  `target/test-manifest`) or it dies with `STATUS_ENTRYPOINT_NOT_FOUND`; this is an
  environment issue, not a code issue.
