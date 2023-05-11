import { state } from "../state/state";
import { Action } from "./actions";

export class MovementSetCenter implements Action {
    startingPoint: Point;

    constructor(x: number, y: number) {
        this.startingPoint = new Point(x, y);
    }
    
    execute(): void {
        state.remote.leftController.startingPoint = this.startingPoint;
    }
}

export class SendMovementDirection implements Action {
    currentPoint: Point;

    constructor(x: number, y: number) {
        this.currentPoint = new Point(x, y);
    }
    
    execute(): void {
        if (!state.remote.leftController.startingPoint) return;
        let radianDirection = state.remote.leftController.startingPoint!.getDirectionInRadians(this.currentPoint);
        state.game.channel!.send(JSON.stringify(radianDirection));
    }
}

export class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    getDirectionInRadians(other: Point): number {
        const deltaX = other.x - this.x;
        const deltaY = other.y - this.y;
        return Math.atan2(deltaY, deltaX);
    }
}