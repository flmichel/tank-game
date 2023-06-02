import { ControllerId } from "../actions/remote";

export interface Remote {
  movementController: Controller | null;
  actionController: Controller | null;
}

export class Controller {
  lastCenter: Point;
  lastPosition: Point;
  touchStartTimestamp: number;

  constructor(initialPosition: Point, touchStartTimestamp: number) {
    this.lastCenter = initialPosition;
    this.lastPosition = initialPosition;
    this.touchStartTimestamp = touchStartTimestamp;
  }
}

export class Point {
  x: number;
  y: number;

  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }
}
