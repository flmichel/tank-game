import { LitElement, html, css } from 'lit-element';
import { customElement } from 'lit/decorators.js';
import { MovementSetCenter, SendMovementDirection } from '../../actions/remote';
import { trigger } from '../../actions/actions';

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
      return html`
        <canvas id="canvas" width="300" height="300"></canvas>
      `;
    }
  
    firstUpdated() {
      const canvas = this.shadowRoot?.querySelector('#canvas') as HTMLCanvasElement;
      const context = canvas?.getContext('2d');
      let isDrawing = false;
      let lastX: number, lastY: number;
      
      if (!context) {
        console.error('Could not get 2D context for canvas element');
        return;
      }
  
      canvas.addEventListener('touchstart', function(event) {
        isDrawing = true;
        lastX = event.touches[0].clientX - canvas.offsetLeft;
        lastY = event.touches[0].clientY - canvas.offsetTop;
        trigger(new MovementSetCenter(lastX, lastY));
      });
  
      canvas.addEventListener('touchmove', function(event) {
        if (isDrawing) {
          const currentX = event.touches[0].clientX - canvas.offsetLeft;
          const currentY = event.touches[0].clientY - canvas.offsetTop;
          trigger(new SendMovementDirection(currentX, currentY));
          context.beginPath();
          context.moveTo(lastX, lastY);
          context.lineTo(currentX, currentY);
          context.stroke();
          lastX = currentX;
          lastY = currentY;
        }
      });
  
      canvas.addEventListener('touchend', function(event) {
        isDrawing = false;
        context.clearRect(0, 0, canvas.width, canvas.height);
      });
    }
  }