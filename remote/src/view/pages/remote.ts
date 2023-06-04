import { LitElement, html, css } from "lit-element";
import { customElement, property } from "lit/decorators.js";
import { CanvasData, ControllerId } from "../../actions/remote";
import { DisplaySettings } from "../../state/state";

@customElement("remote-canvas")
export class CanvasTouch extends LitElement {
  static get styles() {
    return css`
      canvas {
        border: 1px solid black;
      }
    `;
  }

  movementData: CanvasData | null = null;
  actionData: CanvasData | null = null;
  touchedControllers: Map<ControllerId, number> = new Map();

  @property({ reflect: true })
  state!: DisplaySettings;

  render() {
    return html` <div id="remote">
      <canvas
        id="movement"
        width=${this.state.windowWidth / 2}
        height=${this.state.windowHeight / 2}
      ></canvas>
      <canvas
        id="action"
        width=${this.state.windowWidth / 2}
        height=${this.state.windowHeight / 2}
      ></canvas>
    </div>`;
  }

  firstUpdated() {
    const remote = this.shadowRoot?.querySelector("#remote") as HTMLElement;
    new CanvasData(
      ControllerId.MOVEMENT,
      this.shadowRoot!,
      this.touchedControllers
    );
    new CanvasData(
      ControllerId.ACTION,
      this.shadowRoot!,
      this.touchedControllers
    );
  }
}
