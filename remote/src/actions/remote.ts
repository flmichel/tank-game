import { sendStringToGame, sendToGame } from "../api/game";
import { state } from "../state/state";
import { Action, trigger } from "./actions";

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
    if (!state.remote.leftController.startingPoint) return;

    if (this.controllerId == ControllerId.MOVEMENT) {
      let radianDirection =
        state.remote.leftController.startingPoint!.getDirectionInRadians(
          this.currentPoint
        );
      sendToGame({ move: radianDirection });
    } else {
      let radianDirection =
        state.remote.rightController.startingPoint!.getDirectionInRadians(
          this.currentPoint
        );
      sendToGame({ aim: radianDirection });
    }
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
      sendStringToGame("stop");
    } else {
      if (this.lastLocation !== null) {
        let radianDirection =
          state.remote.rightController.startingPoint!.getDirectionInRadians(
            this.lastLocation
          );
        sendToGame({ aim: radianDirection });
        sendStringToGame("shoot");
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

export class CanvasData {
  controllerId: ControllerId;
  context: CanvasRenderingContext2D;
  canvas: HTMLCanvasElement;
  isDrawing: boolean;
  touchIdentifier: number | null = null;
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
    this.isDrawing = false;

    canvas.addEventListener("touchstart", (event) => this.touchStart(event));

    canvas.addEventListener("touchmove", (event) => this.touchMove(event));

    canvas.addEventListener("touchend", (event) => this.touchEnd(event));
  }

  touchStart(event: TouchEvent) {
    event.preventDefault();

    let touch = event.touches[TouchIndex.next()];
    this.touchIdentifier = touch.identifier;
    console.log("start drawing touch", this.touchIdentifier, event.touches);

    this.isDrawing = true;
    this.lastCenter = new Point(
      touch.clientX - this.canvas.offsetLeft,
      touch.clientY - this.canvas.offsetTop
    );
    trigger(new SetControllerCenter(this.controllerId, this.lastCenter));
  }

  touchMove(event: TouchEvent) {
    event.preventDefault();
    if (this.isDrawing) {
      let touch = this.retrieveTouch(event)!;

      if (!this.lastLocation) {
        this.lastLocation = new Point(
          touch.clientX - this.canvas.offsetLeft,
          touch.clientY - this.canvas.offsetTop
        );
      }

      const currentX = touch.clientX - this.canvas.offsetLeft;
      const currentY = touch.clientY - this.canvas.offsetTop;
      const currentLocation = new Point(currentX, currentY);

      let currentTime = new Date().getTime();
      if (!this.lastUpdateTime || this.lastUpdateTime + 80 < currentTime) {
        trigger(new SetControllerDirection(this.controllerId, currentLocation));
        if (
          this.lastLocation!.getDistanceFrom(this.lastCenter!) > 25 &&
          this.controllerId == ControllerId.MOVEMENT
        ) {
          trigger(
            new SetControllerCenter(this.controllerId, this.lastLocation!)
          );
          this.lastCenter = this.lastLocation;
        }
        this.lastUpdateTime = currentTime;
      }

      this.context.beginPath();
      this.context.moveTo(this.lastLocation!.x, this.lastLocation!.y);
      this.context.lineTo(currentX, currentY);
      this.context.stroke();
      this.lastLocation = currentLocation;
    }
  }

  touchEnd(event: TouchEvent) {
    event.preventDefault();
    this.isDrawing = false;
    console.log("touchEnd", this.lastLocation);
    trigger(new StopController(this.controllerId, this.lastLocation));
    TouchIndex.updateIndex(event);
    this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.lastLocation = null;
    this.lastCenter = null;
  }

  retrieveTouch(event: TouchEvent): Touch | undefined {
    const touches = Array.from(event.touches);

    for (const touch of touches) {
      if (touch.identifier === this.touchIdentifier) {
        return touch;
      }
    }

    return undefined;
  }
}

class TouchIndex {
  private static nextIndex: number = 0;

  public static next(): number {
    return TouchIndex.nextIndex++;
  }

  public static updateIndex(event: TouchEvent): void {
    TouchIndex.nextIndex = this.getSmallestMissingIdentifier(event);
  }

  private static getSmallestMissingIdentifier(event: TouchEvent): number {
    const identifiers = Array.from(event.touches).map(
      (touch) => touch.identifier
    );
    console.log("identifiers left", identifiers);

    let smallestMissing = 0;
    while (identifiers.includes(smallestMissing)) {
      smallestMissing++;
    }

    return smallestMissing;
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
