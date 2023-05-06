import { test } from './form';
import {html, css, LitElement} from 'lit';
import { customElement, property} from 'lit/decorators.js';
import './form';

console.log("1");
console.log("2");
console.log("3");
test();

@customElement('simple-greeting')
export class SimpleGreeting extends LitElement {
  static styles = css`p { color: blue }`;

  @property()
  name = 'Somebody';

  render() {
    return html`<p>this is name</p>`;
  }
}