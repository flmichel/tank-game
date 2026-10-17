import { sendToGame } from "../api/game";
import { GamePhase, state } from "../state/state";
import { Action } from "./actions";

let inputs: RTCDataChannel | null = null;
let seq = 0;

// A game step is 1/120 s, so sending faster than that buys nothing and costs airtime: with 16
// phones on one router it is the phones' sends that saturate the Wi-Fi, not the game.
const MIN_SEND_INTERVAL_MS = 1000 / 120;
// Each message is the full stick state, so a lost one only has to be replaced while the sticks are
// still: a moving stick's next change already replaces it.
const IDLE_RESEND_INTERVAL_MS = 1000 / 60;
// Four copies of a resting state in ~50 ms; losing all four is unlikely, and the phone then goes
// quiet instead of repeating itself at 60 Hz for as long as the player holds still.
const IDLE_RESENDS = 3;

let lastSent: Sticks | null = null;
let lastSendTime = 0;
let lastPhase: GamePhase | null = null;
let idleResendsLeft = 0;
let trailingSend: number | null = null;

interface Sticks {
  move: number | null;
  aim: number | null;
}

// The stick state goes out on every change, capped at 120 Hz, and is repeated a few times while
// the sticks are still. Pings measure the network.
export function streamSticks(channel: RTCDataChannel) {
  let timers: number[] = [];
  channel.onopen = () => {
    inputs = channel;
    lastSent = null; // a new channel knows nothing about this phone's sticks
    timers = [
      window.setInterval(resendSticksWhileIdle, IDLE_RESEND_INTERVAL_MS),
      window.setInterval(() => channel.send(JSON.stringify({ ping: performance.now() })), 1000),
    ];
  };
  channel.onclose = () => {
    inputs = null;
    timers.forEach(clearInterval);
    if (trailingSend !== null) {
      clearTimeout(trailingSend);
      trailingSend = null;
    }
  };
  channel.onmessage = (event) => {
    const { pong } = JSON.parse(event.data);
    const display = document.getElementById("ping");
    if (typeof pong === "number" && display) {
      display.textContent = `ping ${Math.round(performance.now() - pong)} ms`;
    }
  };
}

// Called on every stick change. A change that falls inside the cap is not dropped: one trailing
// send carries the newest state as soon as the cap allows.
function sendSticks() {
  if (!canSend()) return;
  const sticks = currentSticks();
  if (lastSent && sticks.move === lastSent.move && sticks.aim === lastSent.aim) return;

  const wait = lastSendTime + MIN_SEND_INTERVAL_MS - performance.now();
  if (wait > 0) {
    if (trailingSend === null) {
      trailingSend = window.setTimeout(() => {
        trailingSend = null;
        sendSticks();
      }, wait);
    }
    return;
  }

  send(sticks);
  idleResendsLeft = IDLE_RESENDS;
}

function resendSticksWhileIdle() {
  if (!canSend() || idleResendsLeft === 0) return;
  if (performance.now() - lastSendTime < IDLE_RESEND_INTERVAL_MS) return; // the sticks are moving
  idleResendsLeft -= 1;
  send(currentSticks());
}

function canSend(): boolean {
  const phase = state.game.state.phase;
  if (phase !== lastPhase) {
    lastPhase = phase;
    // A round starts with the game knowing nothing about this phone, even if a stick is held down
    lastSent = null;
    idleResendsLeft = IDLE_RESENDS;
  }
  return inputs?.readyState === "open" && phase === GamePhase.InGame;
}

function currentSticks(): Sticks {
  const { leftController, rightController } = state.remote;
  return { move: leftController.direction, aim: rightController.direction };
}

function send(sticks: Sticks) {
  lastSent = sticks;
  lastSendTime = performance.now();
  inputs!.send(JSON.stringify({ seq: ++seq, move: sticks.move, aim: sticks.aim }));
}

export class SetControllerCenter implements Action {
  startingPoint: Point;
  controllerId: ControllerId;

  constructor(controllerId: ControllerId, startingPoint: Point) {
    this.startingPoint = startingPoint;
    this.controllerId = controllerId;
  }

  execute(): void {
    if (this.controllerId == ControllerId.MOVEMENT) {
      state.remote.leftController.startingPoint = this.startingPoint;
    } else {
      state.remote.rightController.startingPoint = this.startingPoint;
    }
  }
}

export class SetControllerDirection implements Action {
  currentPoint: Point;
  controllerId: ControllerId;

  constructor(controllerId: ControllerId, currentPoint: Point) {
    this.currentPoint = currentPoint;
    this.controllerId = controllerId;
  }

  execute(): void {
    let controller =
      this.controllerId == ControllerId.MOVEMENT
        ? state.remote.leftController
        : state.remote.rightController;
    if (!controller.startingPoint) return;

    controller.direction = controller.startingPoint.getDirectionInRadians(
      this.currentPoint
    );
  }
}

export class StopController implements Action {
  controllerId: ControllerId;
  lastLocation: Point | null;

  constructor(controllerId: ControllerId, lastLocation: Point | null) {
    this.controllerId = controllerId;
    this.lastLocation = lastLocation;
  }

  execute(): void {
    if (this.controllerId == ControllerId.MOVEMENT) {
      state.remote.leftController.direction = null;
    } else {
      state.remote.rightController.direction = null;
      if (this.lastLocation !== null) {
        let radianDirection =
          state.remote.rightController.startingPoint!.getDirectionInRadians(
            this.lastLocation
          );
        // Reliable channel: the shot carries its aim, it can overtake the latest stick state
        sendToGame({ shoot: radianDirection });
      }
    }
  }
}

export class Point {
  x: number;
  y: number;

  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }

  getDistanceFrom(that: Point): number {
    const distanceX = that.x - this.x;
    const distanceY = that.y - this.y;
    return Math.sqrt(distanceX * distanceX + distanceY * distanceY);
  }

  getDirectionInRadians(other: Point): number {
    const deltaX = other.x - this.x;
    const deltaY = other.y - this.y;
    return Math.atan2(deltaY, deltaX);
  }
}

// Chrome hands over pointer moves as soon as the screen samples them instead of once per frame
// (secure contexts only); elsewhere this falls back to the frame-aligned pointermove.
const POINTER_MOVE = "onpointerrawupdate" in window ? "pointerrawupdate" : "pointermove";

export class CanvasData {
  controllerId: ControllerId;
  context: CanvasRenderingContext2D;
  canvas: HTMLCanvasElement;
  pointerId: number | null = null;
  lastLocation: Point | null = null;
  lastCenter: Point | null = null;
  lastUpdateTime: number | null = null;

  constructor(controllerId: ControllerId, shadowRoot: ShadowRoot) {
    this.controllerId = controllerId;
    const canvas = shadowRoot?.querySelector(
      "#" + controllerId
    ) as HTMLCanvasElement;
    this.canvas = canvas;
    this.context = canvas.getContext("2d")!;

    canvas.addEventListener("pointerdown", (event) => this.pointerDown(event));
    canvas.addEventListener(POINTER_MOVE, (event) =>
      this.pointerMove(event as PointerEvent)
    );
    canvas.addEventListener("pointerup", (event) => this.pointerUp(event));
    canvas.addEventListener("pointercancel", (event) => this.pointerUp(event));
  }

  // Each stick follows one finger
  pointerDown(event: PointerEvent) {
    if (this.pointerId !== null) return;
    event.preventDefault();
    this.pointerId = event.pointerId;
    this.canvas.setPointerCapture(event.pointerId);
    this.lastCenter = this.locate(event);
    new SetControllerCenter(this.controllerId, this.lastCenter).execute();
  }

  pointerMove(event: PointerEvent) {
    if (event.pointerId !== this.pointerId) return;
    const currentLocation = this.locate(event);
    if (!this.lastLocation) {
      this.lastLocation = currentLocation;
    }

    // Steer towards where the finger is about to be (Chrome predicts about a frame ahead), and
    // send before drawing anything. No view update: nothing on the page shows the stick state.
    const predicted = event.getPredictedEvents?.() ?? [];
    const target =
      predicted.length > 0
        ? this.locate(predicted[predicted.length - 1])
        : currentLocation;
    new SetControllerDirection(this.controllerId, target).execute();
    sendSticks();

    // The floating move stick re-centers on the finger, paced to keep its feel
    const currentTime = performance.now();
    if (!this.lastUpdateTime || this.lastUpdateTime + 80 < currentTime) {
      if (
        this.lastLocation.getDistanceFrom(this.lastCenter!) > 25 &&
        this.controllerId == ControllerId.MOVEMENT
      ) {
        new SetControllerCenter(this.controllerId, this.lastLocation).execute();
        this.lastCenter = this.lastLocation;
      }
      this.lastUpdateTime = currentTime;
    }

    this.context.beginPath();
    this.context.moveTo(this.lastLocation.x, this.lastLocation.y);
    this.context.lineTo(currentLocation.x, currentLocation.y);
    this.context.stroke();
    this.lastLocation = currentLocation;
  }

  pointerUp(event: PointerEvent) {
    if (event.pointerId !== this.pointerId) return;
    this.pointerId = null;
    new StopController(this.controllerId, this.lastLocation).execute();
    sendSticks();
    this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.lastLocation = null;
    this.lastCenter = null;
  }

  locate(event: PointerEvent): Point {
    return new Point(
      event.clientX - this.canvas.offsetLeft,
      event.clientY - this.canvas.offsetTop
    );
  }
}

export enum ControllerId {
  MOVEMENT = "movement",
  ACTION = "action",
}

export class UpdateRemoteInput implements Action {
  data: CanvasData;

  constructor(data: CanvasData) {
    this.data = data;
  }

  execute(): void {
    if (this.data.controllerId === ControllerId.MOVEMENT) {
    }
  }
}
