import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { State, state } from "../state/state";
import "./navbar";
import "./pages/homepage";
import "./pages/remote";

export function test() {
  console.log("Testing");
}

@customElement("root-view")
export class View extends LitElement {
  static styles = css`
    p {
      color: blue;
    }
  `;

  @property()
  state: State = state;

  render() {
    return html` <div>
      <nav-bar .state=${this.state.authentication}></nav-bar>
      ${this.renderPage(this.state.route)}
    </div>`;
  }

  renderPage(route: string) {
    switch (route) {
      case "/":
        return html`
          <home-page></home-page>
          <remote-canvas .state=${this.state}></remote-canvas>
        `;
    }
  }
}
