import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Authentication } from "../../state/state";

@customElement("home-page")
export class HomePage extends LitElement {
  static styles = css`
    li {
      color: blue;
    }
  `;

  render() {
    return html` The is the homepage `;
  }
}
