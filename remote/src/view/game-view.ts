import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { GamePhase, State } from "../state/state";
import "./navbar";
import "./pages/homepage";
import "./pages/remote";
import "./pages/game-configuration";
import { DisplayState } from "../state/displayState";

@customElement("game-view")
export class View extends LitElement {
  @property({ type: Object })
  state!: DisplayState;

  render() {
    switch (this.state.gameState.phase) {
      case GamePhase.BeforeNextGame:
        return html`
          <game-configuration
            .state=${this.state.gameState}
          ></game-configuration>
        `;
      case GamePhase.InGame:
        console.log("InGame phase updated");
        return html` <remote-canvas .state=${this.state}></remote-canvas> `;
      default:
        return html` <p>this is not normal</p> `;
    }
  }
}
