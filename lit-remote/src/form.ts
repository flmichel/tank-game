import {html, css, LitElement} from 'lit';
import { customElement, property} from 'lit/decorators.js';

export function test() {
    console.log("Testing");
}

@customElement('test-element')
export class TestElement extends LitElement {
  static styles = css`p { color: blue }`;

  @property()
  name = 'Somebody I use to know';

  render() {
    return html`<p>Hello, ${this.name}!</p>`;
  }
}