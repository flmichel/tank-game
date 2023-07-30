import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { trigger, Route } from "../actions/actions";
import { DisplayState } from "../state/displayState";

@customElement("nav-bar")
export class Navbar extends LitElement {
  static styles = css`
    li {
      color: blue;
    }
  `;

  @property({ type: Object, reflect: true })
  state!: DisplayState;

  render() {
    return html`
      <ul>
        <li><button @click=${() => trigger(new Route("/"))}>Home</button></li>

        <li>
          <button @click=${() => trigger(new Route("/login"))}>Login</button>
        </li>
        <li>
          <button @click=${() => trigger(new Route("/signup"))}>Sign up</button>
        </li>
      </ul>
    `;
  }
}
