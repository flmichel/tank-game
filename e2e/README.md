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
# 5. Run: profile none|good|busy, number of phones. CHROME points at the browser
#    (it defaults to Google Chrome's macOS path).
cd e2e && npm install && CHROME=$(which chromium) node e2e.js good 2
```

## Results

Everything on one machine, over loopback, so the numbers are the game's own cost: the profile's
delay is added on top of it, not measured. Touch → game draws, in ms:

| profile | phones | steps matched | p50 | p90 | p99 | join p50 |
| ------- | ------ | ------------- | ---- | ---- | ---- | -------- |
| none    | 2      | 300/300       | 16.3 | 33.2 | 34.5 | 1.2 s    |
| good    | 2      | 300/300       | 19.5 | 33.3 | 49.9 | 1.2 s    |
| busy    | 2      | 300/300       | 23.1 | 38.4 | 61.0 | 1.2 s    |
| none    | 16     | 300/300       | 13.9 | 30.5 | 34.8 | 4.4 s    |
| good    | 16     | 300/300       | 17.8 | 31.8 | 49.3 | 6.4 s    |
| busy    | 16     | 300/300       | 22.0 | 42.5 | 51.8 | 3.5 s    |

16 phones cost nothing in latency: the aim line still lands within a frame or two. Joining is what
16 phones make slow — each one waits for its own offer/answer round trip (see the trickle ICE item
in HANDOFF.md).

A run leaves no browsers behind; if one is killed mid-run, clear them with
`pkill -f puppeteer_dev_chrome_profile`.
