import {html, css, LitElement} from 'lit';
import { customElement, property} from 'lit/decorators.js';
import { Authentication, AuthenticationState, State } from '../../state/state';
import { trigger, Route } from '../../actions/actions';

@customElement('homepage')
export class Navbar extends LitElement {
  static styles = css`li { color: blue }`;

  @property()
  state!: Authentication;

  render() {
    return html`
      <ul>
        <li><button @click=${() => trigger(new Route('/'))}>Home</button></li>
        ${(this.state.authenticationState === AuthenticationState.LoggedIn) ?
          html`<li><button @click=${() => trigger(new Route('/settings'))}>Account Settings</button></li>` : 
          html`
            <li><button @click=${() => trigger(new Route('/login'))}>Login</button></li>
            <li><button @click=${() => trigger(new Route('/signup'))}>Sign up</button></li>
          `
        }
      </ul>
  `;
  }
}