import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { trigger, Route } from "../actions/actions";
import { StyledElement } from "./style/styled-element";

@customElement("nav-bar")
export class Navbar extends LitElement {
  static styles = [
    StyledElement.styles,
    css`
      ul {
        list-style: none;
        display: flex;
        background-color: #000000;
        justify-content: center;
        align-items: center;
      }

      li {
        margin: 0;
        padding: 10px;
      }

      li img {
        width: 40px;
        height: 40px;
        cursor: pointer;
      }
    `,
  ];

  render() {
    return html`
      <ul>
        <li><img src="emoji-icon.png" @click=${() =>
          trigger(new Route("/"))}></button></li>

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
