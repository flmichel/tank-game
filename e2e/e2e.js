// End-to-end latency test: the real web game and real remotes in headless Chrome, the real server,
// real WebRTC. Wi-Fi is simulated at each phone's data channel (delay, jitter, loss).
//
// usage: node e2e.js <none|good|busy> <players>
//
// Phone 1 sweeps its aim stick through known angles. We time each step from the touch event
// (t0) to the remote sending it (t1) to the game drawing an aim line at that angle (t2).
// Other phones drive their movement stick continuously, as load.
const puppeteer = require("puppeteer-core");
const { performance } = require("node:perf_hooks");

const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const GAME_URL = "http://localhost:8081/?server=ws://localhost:5050/ws";
const REMOTE_URL = "http://localhost:8080/?room-id=";
const ARGS = [
  "--disable-features=WebRtcHideLocalIpsWithMdns", // real host candidates, not .local names
  "--disable-background-timer-throttling",
  "--disable-renderer-backgrounding",
  "--disable-backgrounding-occluded-windows",
];

// One-way phone → game delay in ms: base + lognormal jitter (median, sigma), rare spikes, loss.
// Assumed profiles, not measurements: "good" = quiet home Wi-Fi 5, phone near the router, game
// machine wired; "busy" = other devices streaming, game machine also on Wi-Fi.
const PROFILES = {
  none: { base: 0, median: 0, sigma: 0, loss: 0, spike: 0, spikeMin: 0, spikeMax: 0 },
  good: { base: 1.5, median: 1, sigma: 0.77, loss: 0.002, spike: 0.01, spikeMin: 10, spikeMax: 30 },
  busy: { base: 4, median: 3, sigma: 0.9, loss: 0.01, spike: 0.03, spikeMin: 20, spikeMax: 80 },
};

const SWEEP_STEPS = 300;
const SWEEP_INTERVAL_MS = 40;
const angleOf = (k) => ((k * 0.07) % 6) - 3; // distinct for 85 steps, away from the ±π seam

const now = () => performance.timeOrigin + performance.now();
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function gameProbe() {
  window.__aim = [];
  window.__frames = [];
  const proto = CanvasRenderingContext2D.prototype;
  const { moveTo, lineTo, stroke } = proto;
  let from, to, last;
  proto.moveTo = function (x, y) { from = [x, y]; return moveTo.call(this, x, y); };
  proto.lineTo = function (x, y) { to = [x, y]; return lineTo.call(this, x, y); };
  proto.stroke = function () {
    if (this.strokeStyle === "#00ff00" && from && to) {
      const angle = Math.atan2(to[1] - from[1], to[0] - from[0]);
      if (angle !== last) {
        last = angle;
        window.__aim.push([performance.timeOrigin + performance.now(), angle]);
      }
    }
    return stroke.call(this);
  };
  const frame = (t) => { window.__frames.push(t); requestAnimationFrame(frame); };
  requestAnimationFrame(frame);
}

function phoneShaper(profile) {
  window.__sent = [];
  const send = RTCDataChannel.prototype.send;
  let reliableAt = 0;
  const gauss = () => {
    let u = 0, v = 0;
    while (!u) u = Math.random();
    while (!v) v = Math.random();
    return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
  };
  const delay = () => {
    if (!profile.median) return 0;
    let d = profile.base + profile.median * Math.exp(profile.sigma * gauss());
    if (Math.random() < profile.spike) d += profile.spikeMin + Math.random() * (profile.spikeMax - profile.spikeMin);
    return d;
  };
  RTCDataChannel.prototype.send = function (data) {
    const sentAt = performance.timeOrigin + performance.now();
    if (typeof data === "string" && data.startsWith('{"seq"')) window.__sent.push([sentAt, JSON.parse(data).aim]);
    const channel = this;
    let at = sentAt + delay();
    if (channel.ordered) {
      // Reliable channel: a lost packet is retransmitted (~200 ms), order is kept
      if (Math.random() < profile.loss) at += 200;
      at = reliableAt = Math.max(reliableAt, at);
    } else if (Math.random() < profile.loss) {
      return; // unreliable channel: the packet is simply gone
    }
    const wait = at - sentAt;
    if (wait <= 0) return send.call(channel, data);
    setTimeout(() => channel.readyState === "open" && send.call(channel, data), wait);
  };
}

async function openGame() {
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ARGS });
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 720 });
  await page.evaluateOnNewDocument(gameProbe);
  const roomId = new Promise((resolve) =>
    page.on("console", (message) => {
      const match = message.text().match(/room-id=([A-Za-z0-9_-]+)/);
      if (match) resolve(match[1]);
    })
  );
  await page.goto(GAME_URL);
  return { browser, page, roomId: await roomId };
}

async function joinPhone(browser, roomId, index, profile) {
  const context = await browser.createBrowserContext(); // own localStorage, so its own player id
  const page = await context.newPage();
  await page.setViewport({ width: 390, height: 844, isMobile: true, hasTouch: true, deviceScaleFactor: 2 });
  await page.evaluateOnNewDocument(phoneShaper, profile);
  const opened = new Promise((resolve) =>
    page.on("console", (message) => message.text().includes("channel with room has opened") && resolve())
  );
  const started = now();
  await page.goto(REMOTE_URL + roomId);
  await opened;
  const joinMs = now() - started;
  await page.locator('pierce/input[placeholder="Enter username"]').fill(`p${index}`);
  const readyButton = `document.querySelector("root-view")?.shadowRoot?.querySelector("game-view")?.shadowRoot
    ?.querySelector("game-configuration")?.shadowRoot?.querySelector("button")`;
  // Done once the button says Ready, or is gone because the last player's ready started the game
  for (let attempt = 0; (await page.evaluate(`${readyButton}?.textContent.trim()`)) === "Not Ready"; attempt++) {
    if (attempt === 10) throw new Error(`phone ${index} could not get ready`);
    await sleep(100); // let the name's re-render settle, or the click can be lost
    await page.evaluate(`${readyButton}.click()`);
    // A second click before this one shows would toggle back to not ready
    await page
      .waitForFunction(`${readyButton}?.textContent.trim() !== "Not Ready"`, { timeout: 2000 })
      .catch(() => {});
  }
  return { page, joinMs, cdp: await page.createCDPSession() };
}

async function stickCenter(page, id) {
  const box = await (await page.waitForSelector(`pierce/#${id}`, { timeout: 30000 })).boundingBox();
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function sweepAim(phone) {
  const center = await stickCenter(phone.page, "action");
  await phone.cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ ...center, id: 1 }] });
  await sleep(100);
  const dispatched = [];
  for (let k = 0; k < SWEEP_STEPS; k++) {
    const angle = angleOf(k);
    const point = { x: center.x + 80 * Math.cos(angle), y: center.y + 80 * Math.sin(angle), id: 1 };
    dispatched.push([now(), k]);
    await phone.cdp.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [point] });
    await sleep(SWEEP_INTERVAL_MS - 10 + Math.random() * 20); // decorrelated from the game's frames
  }
  await phone.cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  return dispatched;
}

async function driveMovement(phone, until) {
  const center = await stickCenter(phone.page, "movement");
  await phone.cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ ...center, id: 2 }] });
  for (let k = 0; now() < until; k++) {
    const angle = k * 0.1;
    const point = { x: center.x + 60 * Math.cos(angle), y: center.y + 60 * Math.sin(angle), id: 2 };
    await phone.cdp.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [point] });
    await sleep(16);
  }
}

// First event within 1 s after `time` showing step k or newer: an older state that arrives late is
// ignored by the game, and the screen then shows a newer one, which is what the player sees.
const firstShowing = (events, time, k, tolerance) =>
  events.find(
    ([t, value]) =>
      t >= time && t < time + 1000 && value !== null &&
      Array.from({ length: 20 }, (_, j) => angleOf(k + j)).some((angle) => Math.abs(value - angle) < tolerance)
  );

function stats(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const at = (q) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
  return { p50: at(0.5), p90: at(0.9), p99: at(0.99), max: sorted[sorted.length - 1] };
}

const fmt = ({ p50, p90, p99, max }) =>
  `p50 ${p50.toFixed(1)}  p90 ${p90.toFixed(1)}  p99 ${p99.toFixed(1)}  max ${max.toFixed(1)}`;

async function main() {
  const [profileName = "good", playersArg = "2"] = process.argv.slice(2);
  const profile = PROFILES[profileName];
  const players = Number(playersArg);

  const game = await openGame();
  const phonesBrowser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ARGS });
  try {
    await measure(game, phonesBrowser, profileName, profile, players);
  } finally {
    await phonesBrowser.close();
    await game.browser.close();
  }
}

async function measure(game, phonesBrowser, profileName, profile, players) {
  const phones = await Promise.all(
    Array.from({ length: players }, (_, index) => joinPhone(phonesBrowser, game.roomId, index + 1, profile))
  );
  await Promise.all(phones.map((phone) => stickCenter(phone.page, "action"))); // everyone is in game

  const [aimer, ...others] = phones;
  const sweepEnds = now() + SWEEP_STEPS * (SWEEP_INTERVAL_MS + 5) + 500;
  const [dispatched] = await Promise.all([sweepAim(aimer), ...others.map((phone) => driveMovement(phone, sweepEnds))]);
  await sleep(500);

  const sent = await aimer.page.evaluate(() => window.__sent);
  const drawn = await game.page.evaluate(() => window.__aim);
  const frames = await game.page.evaluate(() => window.__frames);

  const remote = [], network = [], total = [];
  for (const [t0, k] of dispatched) {
    const send = firstShowing(sent, t0, k, 0.01);
    const draw = send && firstShowing(drawn, send[0], k, 0.02);
    if (!draw) continue;
    remote.push(send[0] - t0);
    network.push(draw[0] - send[0]);
    total.push(draw[0] - t0);
  }
  const intervals = frames.slice(1).map((t, i) => t - frames[i]);

  console.log(`profile ${profileName}, ${players} players, ${total.length}/${dispatched.length} aim steps matched`);
  console.log(`  join (page load → channel open): ${fmt(stats(phones.map((p) => p.joinMs)))} ms`);
  console.log(`  touch → remote sends:            ${fmt(stats(remote))} ms`);
  console.log(`  remote sends → game draws:       ${fmt(stats(network))} ms`);
  console.log(`  touch → game draws (total):      ${fmt(stats(total))} ms`);
  console.log(`  game frame interval:             ${fmt(stats(intervals))} ms, ${intervals.filter((i) => i > 25).length} frames > 25 ms`);

}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
