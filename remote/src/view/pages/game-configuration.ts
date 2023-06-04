import { LitElement, html, css } from "lit-element";
import { customElement, property } from "lit/decorators.js";
import { CanvasData, ControllerId } from "../../actions/remote";
import { DisplaySettings, GameState } from "../../state/state";
import { trigger } from "../../actions/actions";
import {
  ToggleReady,
  UpdatePlayerName,
} from "../../actions/game-configuration";

@customElement("game-configuration")
export class CanvasTouch extends LitElement {
  @property({ reflect: true })
  state!: GameState;

  render() {
    console.log(this.state);
    return html`
      <input
        type="text"
        placeholder="Enter username"
        @input=${(event: any) =>
          trigger(new UpdatePlayerName(event.target.value))}
      />
      <button @click=${() => trigger(new ToggleReady())}>
        ${this.state.isReady ? "Ready" : "Not Ready"}
      </button>
    `;
  }
}
