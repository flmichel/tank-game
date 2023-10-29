import { LitElement, html, css, CSSResultGroup } from "lit";
import { customElement } from "lit/decorators.js";

export abstract class StyledElement extends LitElement {
  static styles = css`
    button {
      color: black;
      font-size: 30px;
      background-color: #ffbe0a;
      border-radius: 10px;
    }

    button:hover {
      background-color: #ffdf9a;
    }

    input {
      font-size: 20px;
      border-radius: 10px;
    }

    p {
      font-size: 20px;
    }
  `;
}
