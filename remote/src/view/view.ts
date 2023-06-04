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

  @property()
  state: DisplayState = computeDisplayState(state);

  updateState(newState: State) {
    this.state = computeDisplayState(state);
    this.requestUpdate();
  }

  render() {
    return html` <div>
      <nav-bar .state=${this.state}></nav-bar>
      <game-view .state=${this.state}></game-view>
      ${this.renderPage(this.state.route)}
    </div>`;
  }

  renderPage(route: string) {
    console.log("rerendering page (root view)", this.state.gameState.isReady);
    switch (route) {
      case "/":
        return html`
          <home-page></home-page>
          <game-view .state=${this.state}></game-view>
        `;
    }
  }
}
