# End-to-end latency test

Runs the real web game and simulated phones (the real remote) in headless Chrome, against the real
server, over real WebRTC. Wi-Fi is simulated at each phone's data channel (delay, jitter, loss).
It times each aim-stick step from the touch event to the game drawing the aim line.

```bash
# 1. Postgres for the server
docker run -d --name tank-e2e-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=tank-game \
  -p 127.0.0.1:5432:5432 postgres:15.2-alpine
# 2. Server, games WebSocket on 5050 (macOS AirPlay holds 5000)
mkdir -p /tmp/tank-server && cp e2e/server-configuration.yaml /tmp/tank-server/configuration.yaml
(cd /tmp/tank-server && SQLX_OFFLINE=true cargo run --manifest-path "$OLDPWD/Cargo.toml" -p server)
# 3. Remote built against localhost, served on 8080
#    (put API_BASE_URL=http://localhost:8000/api in remote/.env first)
(cd remote && npx webpack --mode production && python3 -m http.server 8080 -d public)
# 4. Web game, served on 8081
(cd game-web && trunk build --release && python3 -m http.server 8081 -d dist)
# 5. Run: profile none|good|busy, number of phones
cd e2e && npm install && node e2e.js good 2
```

First result (no simulated Wi-Fi, 2 phones): touch → game draws p50 ~15 ms, p90 ~27 ms, p99 ~30 ms.
Still to do: the good/busy Wi-Fi and 16-phone runs were flaky (a Ready click lost right after
typing the name; leftover headless Chromes from failed runs). Both are fixed in the script but
not re-verified.
