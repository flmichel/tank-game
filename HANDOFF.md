# Handoff: tank-game

## Where things are
- Branch `game-core`: all current work, ready to build and test. `master` is untouched, no PRs.
- Branch `deps-upgrade` (29b50569): all deps bumped to latest, but on the OLD pre-split layout
  (`game/`). Must be ported onto the new crates (see Tasks).

## Architecture (ports & adapters)
- `game-core`: pure game logic (specs ECS, systems, map, render logic). No IO, no async, must
  compile for `wasm32-unknown-unknown`. Ports:
  - in: `Game::handle(MessageToGame)` (`MessageToGame::from_remote` maps phone input)
  - out: `Game::drain_outbox() -> Vec<(socket_id, MessageToRemote)>`
  - drawing: `Renderer` trait (clear / fill_rect / draw_line / draw_sprite / draw_text)
  - time: `Game::tick(dt)` runs fixed 120 Hz steps (`STEPS_PER_SECOND`)
  - `game_core::protocol` holds the server signaling types and Ping/Pong
- `game-native`: SDL2 + tokio + webrtc-rs shell; vsync-paced loop.
- `game-web`: wasm-bindgen shell: canvas 2D (desynchronized), browser WebSocket and
  RTCPeerConnection. `cd game-web && trunk serve` serves on port 8081;
  `?server=ws://host:port/ws` overrides the signaling server.
- `server`: axum HTTP API on 8000 (phones POST SDP offers to `/api/game/{id}`); games WebSocket
  on 5000.
- `remote`: TypeScript/lit phone controller (webpack); `API_BASE_URL` in `remote/.env`.

## Wire protocol (deploy remote + game together)
- `"channel"` (reliable):
  - phone → game: `{"player-id":id}`, `{"set-name":n}`, `"ready"`, `"not-ready"`, `{"shoot":radians}`
  - game → phone: `"GameStart"`, `"GameOver"`
- `"inputs"` (unordered, maxRetransmits 0):
  - `{"seq":n,"move":rad|null,"aim":rad|null}` on every stick change, plus a 60 Hz resend
  - `{"ping":t}` → `{"pong":t}`, answered by the shell, not the game loop; the phone shows the RTT

## Setup (macOS)
```bash
brew install rustup sdl2 sdl2_image sdl2_ttf trunk
export PATH=/opt/homebrew/opt/rustup/bin:$PATH LIBRARY_PATH=/opt/homebrew/lib SQLX_OFFLINE=true
rustup default stable && rustup target add wasm32-unknown-unknown
```
- macOS AirPlay Receiver holds port 5000: turn it off, or run the server with
  `e2e/server-configuration.yaml` (games port 5050).
- `remote/.env` and the QR URL in `game-core/src/game.rs` still hard-code `192.168.178.24`.

## Verify
```bash
cargo test -p game-core      # 3 tests: game, inputs, rounds
cargo test -p server         # room-map timeout test
cargo build -p game-native
cargo build -p game-web --target wasm32-unknown-unknown
(cd remote && npm ci && npm run build)
```

## Results so far
- Input lag (old core): 24/42/51 s after 60 s with 2/4/8 players. Now: ≤ 1 step.
- Native CPU with no players: 88% of a core → 1.5% (removed busy loops).
- E2E (`e2e/`, no simulated Wi-Fi, 2 phones): touch → aim line drawn p50 ~15 ms, p99 ~30 ms.
- Estimated touch → screen: ~35–70 ms at 60 Hz, ~25–50 ms with Ethernet, 5 GHz and a 120 Hz
  display.

## Tasks, in priority order
1. **Finish the E2E runs** (`e2e/README.md`): `node e2e.js good|busy 2|16`. Recent fixes, not
   yet re-verified:
   - Ready click lost right after typing the name: fixed with a 100 ms settle and a retry.
   - Leftover headless Chromes from failed runs: browsers now close in `finally`; kill strays
     with `pkill -f puppeteer_dev_chrome_profile`.
2. **16 players.** Blockers:
   - The map has 10 spawns (`game-native/assets/map.txt`).
   - The lobby list overflows (`render_before_game`: y = 200 + 60 per player).
   - Cap each phone's sends (~120 Hz, resend only when idle) so 16 phones don't flood one
     Wi-Fi router.
3. **Port `deps-upgrade`** onto `game-core` / `game-native` / `game-web` / `server` / `remote`:
   - `rand` 0.10: `random_range`
   - axum 0.8: `{id}` routes, `axum::serve`
   - `webrtc` stays at 0.17, `typescript` stays at 6
4. **Faster joining:**
   - Trickle ICE over the existing WebSocket; today both sides wait for full ICE gathering.
   - Use one reliable STUN server; the remote lists 4.
5. **Remaining review items:**
   - Disconnect handling: close peer connections, add a player-left message.
   - Collision on a grid.
   - Split the `HandleInputs` system.
   - The QR code is drawn transposed (`room_code.rs`, `x*w+y`).
   - Hard-coded URLs.
   - Native driver frame queue.
   - Screen Wake Lock on the remote.
6. **Real-phone test:** 240 fps slow-motion video of finger + screen, and check the ping display.

## User preferences
- Commit and push only when asked.
- Minimal, root-cause changes.
- Short, action-first answers.
