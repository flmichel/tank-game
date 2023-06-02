/*import { LitElement, html, css } from "lit-element";
import { customElement } from "lit/decorators.js";
import {
  SetControllerCenter,
  SetControllerDirection,
} from "../../actions/remote";
import { trigger } from "../../actions/actions";

@customElement("canvas-touch")
export class CanvasTouch extends LitElement {
  static get styles() {
    return css`
      canvas {
        border: 1px solid black;
      }
    `;
  }

  render() {
    return html` <canvas id="canvas" width="300" height="300"></canvas> `;
  }

  firstUpdated() {
    const canvas = this.shadowRoot?.querySelector(
      "#canvas"
    ) as HTMLCanvasElement;
    const context = canvas?.getContext("2d");
    console.log("context: ", context);
    console.log("canvasId: " + "working one", canvas);
    let isDrawing = false;
    let lastX: number, lastY: number;
    let lastCenterX: number, lastCenterY: number;
    let lastUpdateTime: number;

    if (!context) {
      console.error("Could not get 2D context for canvas element");
      return;
    }

    canvas.addEventListener("touchstart", (event: TouchEvent) => {
      event.preventDefault();
      isDrawing = true;
      lastX = event.touches[0].clientX - canvas.offsetLeft;
      lastY = event.touches[0].clientY - canvas.offsetTop;
      lastCenterX = lastX;
      lastCenterY = lastY;
      trigger(new SetControllerCenter(lastCenterX, lastCenterY));
    });

    canvas.addEventListener("touchmove", (event: TouchEvent) => {
      event.preventDefault();
      if (isDrawing) {
        const currentX = event.touches[0].clientX - canvas.offsetLeft;
        const currentY = event.touches[0].clientY - canvas.offsetTop;

        let currentTime = new Date().getTime();
        if (!lastUpdateTime || lastUpdateTime + 100 < currentTime) {
          trigger(new SetControllerDirection(currentX, currentY));
          if (distance(lastX, lastY, lastCenterX, lastCenterY) > 15) {
            trigger(new SetControllerCenter(lastX, lastY));
            lastCenterX = lastX;
            lastCenterY = lastY;
          }
          lastUpdateTime = currentTime;
        }

        context.beginPath();
        context.moveTo(lastX, lastY);
        context.lineTo(currentX, currentY);
        context.stroke();
        lastX = currentX;
        lastY = currentY;
      }
    });

    canvas.addEventListener("touchend", (event: TouchEvent) => {
      event.preventDefault();
      isDrawing = false;
      context.clearRect(0, 0, canvas.width, canvas.height);
    });
  }
}

function distance(x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  console.log("P1", x1, x2);
  console.log("P2", y1, y2);
  console.log(Math.sqrt(dx * dx + dy * dy));
  return Math.sqrt(dx * dx + dy * dy);
}
*/
