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
  - drawing: `Renderer` trait (clear / fill_rect / draw_line / draw_sprite / draw_text); text is
    24 px tall in every shell and (x, y) is its top left
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
  - `{"seq":n,"move":rad|null,"aim":rad|null}` on every stick change, capped at 120 Hz (one
    trailing send carries a change that falls inside the cap), then repeated 3 times at 60 Hz
    while the sticks are still, after which the phone goes quiet
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

## Setup (NixOS / WSL)
```bash
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#gcc nixpkgs#pkg-config nixpkgs#lld \
  nixpkgs#SDL2 nixpkgs#SDL2_image nixpkgs#SDL2_ttf nixpkgs#trunk
export SQLX_OFFLINE=true LIBRARY_PATH=$LD_LIBRARY_PATH
```
- nixpkgs' `rustc` already ships the `wasm32-unknown-unknown` std; `lld` is what the wasm target
  links with. `trunk` downloads its own matching `wasm-bindgen`.
- For the e2e, `CHROME=$(which chromium)` and a Postgres on a free port (another Postgres may
  already hold 5432; change `port:` in the copied `server-configuration.yaml` to match).

## Verify
```bash
cargo test -p game-core      # 5 tests: game, inputs, rounds, full_game (lobby + spawns)
cargo test -p server         # room-map timeout test
cargo build -p game-native
cargo build -p game-web --target wasm32-unknown-unknown
(cd remote && npm ci && npm run build)
```

## Results so far
- Input lag (old core): 24/42/51 s after 60 s with 2/4/8 players. Now: ≤ 1 step.
- Native CPU with no players: 88% of a core → 1.5% (removed busy loops).
- E2E: all 6 profile × player-count runs pass, 300/300 aim steps matched. Touch → aim line drawn
  p50 14–23 ms, p99 35–61 ms, and **16 phones cost no more latency than 2**. Full table in
  `e2e/README.md`.
- Joining is the slow part: p50 1.2 s with 2 phones, 3.5–6.4 s with 16.
- Estimated touch → screen: ~35–70 ms at 60 Hz, ~25–50 ms with Ethernet, 5 GHz and a 120 Hz
  display.

## Tasks, in priority order
1. **Faster joining** (now the worst number):
   - Trickle ICE; today both sides send one signaling message and so must gather candidates first.
     The phone POSTs its offer and gets the answer in the HTTP response, so trickling needs a way
     back to the phone (a WebSocket, or polling) — a protocol change across server + remote +
     both shells.
   - `stun.l.google.com:19302` never finished gathering in headless Chromium here: host and srflx
     candidates arrived, then the state stayed `gathering` past 30 s, so every join hit the
     server's 10 s answer timeout. `game-web` now caps the wait at 1 s
     (`ICE_GATHERING_TIMEOUT_MS`) and answers with what it has. Decide whether a LAN-only game
     wants STUN at all: with no ICE servers, gathering completes in ~120 ms. `game-native` waits
     on `gathering_complete_promise()` and has the same exposure. The remote lists 4 STUN servers.
2. **A round starts as soon as the players already in the lobby are all ready**, so with 16 phones
   scanning the QR at their own pace the first two to be ready start a round and the rest wait it
   out (a round start also resets everyone to not ready). The e2e now readies every phone only
   after all of them have joined. Needs a lobby policy: a countdown, a host, or a minimum wait.
3. **Port `deps-upgrade`** onto `game-core` / `game-native` / `game-web` / `server` / `remote`:
   - `rand` 0.10: `random_range`
   - axum 0.8: `{id}` routes, `axum::serve`
   - `webrtc` stays at 0.17, `typescript` stays at 6
4. **Remaining review items:**
   - Disconnect handling: close peer connections, add a player-left message.
   - Collision on a grid.
   - Split the `HandleInputs` system.
   - Hard-coded URLs (`remote/.env`, the QR URL in `game-core/src/game.rs`, `SERVER_PORT` in
     `game-web`).
   - Native driver frame queue.
   - Screen Wake Lock on the remote.
5. **Real-phone test:** 240 fps slow-motion video of finger + screen, and check the ping display.

## Done since the last handoff
- E2E: all runs finished (see above). The 16-phone failure was the lobby policy in task 2, not the
  Ready click; no leftover browsers. `CHROME` now selects the browser.
- 16 players: the map has 16 spawns, the lobby roster shrinks its rows and wraps into columns to
  fit any resolution, and each phone's sends are capped (see the wire protocol). `Map::parse` takes
  its dimensions from the map file instead of a hard-coded 16 × 9.
- The QR code was drawn transposed (`x * width + y` over row-major modules) and re-rendered for
  every one of its ~1000 modules every frame. Both fixed.

## User preferences
- Commit and push only when asked.
- Minimal, root-cause changes.
- Short, action-first answers.
