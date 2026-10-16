import { sendToGame } from "../api/game";
import { GamePhase, state } from "../state/state";
import { Action } from "./actions";

let inputs: RTCDataChannel | null = null;
let seq = 0;

// The stick state goes out on every change, plus a 60 Hz resend: each message is the full state,
// so a lost one (even a release) is replaced by the next within ~17 ms. Pings measure the network.
export function streamSticks(channel: RTCDataChannel) {
  let timers: number[] = [];
  channel.onopen = () => {
    inputs = channel;
    timers = [
      window.setInterval(sendSticks, 1000 / 60),
      window.setInterval(() => channel.send(JSON.stringify({ ping: performance.now() })), 1000),
    ];
  };
  channel.onclose = () => {
    inputs = null;
    timers.forEach(clearInterval);
  };
  channel.onmessage = (event) => {
    const { pong } = JSON.parse(event.data);
    const display = document.getElementById("ping");
    if (typeof pong === "number" && display) {
      display.textContent = `ping ${Math.round(performance.now() - pong)} ms`;
    }
  };
}

function sendSticks() {
  if (inputs?.readyState !== "open" || state.game.state.phase !== GamePhase.InGame) return;
  const { leftController, rightController } = state.remote;
  inputs.send(
    JSON.stringify({
      seq: ++seq,
      move: leftController.direction,
      aim: rightController.direction,
    })
  );
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
