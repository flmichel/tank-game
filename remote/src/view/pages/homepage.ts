import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Authentication } from "../../state/state";
import { DisplayState } from "../../state/displayState";
import { StyledElement } from "../style/styled-element";
import { trigger } from "../../actions/actions";
import { EnterGame } from "../../actions/game-configuration";

@customElement("home-page")
export class HomePage extends LitElement {
  static styles = [
    StyledElement.styles,
    css`
      form,
      div {
        text-align: center;
      }
      input {
        margin: 10px 0;
      }
    `,
  ];

  @property({ type: Object })
  state!: DisplayState;

  render() {
    return html`
      <nav-bar .state=${this.state}></nav-bar>
      ${this.renderPage(this.state.route)}
    `;
  }

  renderPage(route: string) {
    switch (route) {
      case "/":
        return html`
          <form
            @submit=${() =>
              trigger(
                new EnterGame(
                  this.shadowRoot!.getElementById("inputField")!.innerHTML
                )
              )}
          >
            <input
              type="text"
              id="inputField"
              placeholder="Enter game room ID"
              required
            />
            <br />
            <button type="submit">Join room</button>
          </form>
          <div>
            <p>Or</p>
            <button>Scan QR Code</button>
          </div>
        `;
    }
  }
}
