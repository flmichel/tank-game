import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { State, state } from "../state/state";
import "./navbar";
import "./pages/homepage";
import "./game-view";
import { DisplayState, computeDisplayState } from "../state/displayState";

@customElement("root-view")
export class View extends LitElement {
  static styles = css`
    p {
      color: blue;
    }
  `;

  @property({ type: Object })
  state: DisplayState = computeDisplayState(state);

  updateState(newState: State) {
    this.state = computeDisplayState(newState);
    this.requestUpdate();
  }

  render() {
    if (this.state.isInRoom) {
      return html`<game-view .state=${this.state}></game-view>`;
    } else {
      return html` <home-page .state=${this.state}></home-page> `;
    }
  }
}
