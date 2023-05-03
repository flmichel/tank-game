import { Actions } from "../actions.js";

export let render = (state) => {
  var output = navbar(state);
  switch (state.route) {
    case "/":
      output += `
        <a>is connected ${state.webRTC.isChannelOpen}</a>
        <button onclick="trigger('${Actions.Decrement}', {})"">decrease</button>
        <a>${state.counter}</a>
        <button onclick="trigger('${Actions.Increment}', {})">Increase</button>
        `;
      break;
    case "/login":
      output += loginPage(state.email);
      break;
    case "/signup":
      output += signupPage();
      break;
  }

  display(output);

  function navbar(state) {
    let navbar = `<ul>
      <li><button onclick="trigger('${Actions.Route}', '/')">Home</button></li>
      <li><button onclick="trigger('${Actions.Route}', '/login')">Login</button></li>
      <li><button onclick="trigger('${Actions.Route}', '/signup')">Sign up</button></li>
    </ul>
    `;
    if (state.login) {
      navbar += `<p>Hello ${state.email}<p>`;
    }
    return navbar;
  }

  function loginPage() {
    return `
        <form onsubmit="event.preventDefault();trigger('${Actions.TryLogin}', new FormData(event.target))">
            <h3>Login</h3>
          <button>Login with Google</button></br>
            <input name="email" type="email" placeholder="Email"></br>
            <input name="password" type="password" placeholder="Password" />
            <input name="rememberMe" type="checkbox"/>
            Remember me
          <input type="submit" value="Login">
          <a href="#">Forgot Password</a>
          <p>( or <button onClick="trigger('${Actions.Route}', '/signup')"> Sign up </button> )</p>
        </form>
    `;
  }

  function signupPage() {
    return `
        <form onsubmit="event.preventDefault();trigger('${Actions.TryRegister}', new FormData(event.target))">
            <h3>Sign up</h3>
          <button>Sign up with Google</button></br>
          <input name="email" type="email" placeholder="Email"></br>
          <input name="username" placeholder="Username"></br>
          <input name="password" type="password" placeholder="Password" /></br>
          <input type="submit" value="Sign up">
          <p>( or <button onClick="trigger('${Actions.Route}', '/login')"> Login </button> )</p>
        </form>
    `;
  }
};

let display = (nextView) => {
  let view = document.getElementsByTagName("body")[0];
  view.innerHTML = nextView;
};
