import { LitElement, html, css } from "lit-element";
import { customElement, property } from "lit/decorators.js";
import { CanvasData, ControllerId } from "../../actions/remote";
import { DisplayState } from "../../state/displayState";

@customElement("game-remove")
export class GameRemote extends LitElement {
  static get styles() {
    return css`
      #remote {
        display: flex;
      }
      canvas {
        border: 1px solid black;
      }
    `;
  }

  movementData: CanvasData | null = null;
  actionData: CanvasData | null = null;
  touchedControllers: Map<ControllerId, number> = new Map();

  @property({ type: Object, reflect: true })
  state!: DisplayState;

  render() {
    return html` <div id="remote">
      <canvas
        id="movement"
        width=${this.state.displaySettings.windowWidth / 2}
        height=${this.state.displaySettings.windowHeight}
      ></canvas>
      <canvas
        id="action"
        width=${this.state.displaySettings.windowWidth / 2}
        height=${this.state.displaySettings.windowHeight}
      ></canvas>
    </div>`;
  }

  firstUpdated() {
    const remote = this.shadowRoot?.querySelector("#remote") as HTMLElement;
    new CanvasData(ControllerId.MOVEMENT, this.shadowRoot!);
    new CanvasData(ControllerId.ACTION, this.shadowRoot!);
  }
}
