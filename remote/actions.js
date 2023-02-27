import { Requests } from "./api/requests.js";
import state from "./state.js";
import { render } from "./view/view.js";
import { execute } from "./api/requests.js";

export const Actions = {
  Increment: "increment",
  Decrement: "decrement",
  Route: "route",
  RouteBack: "routeBack",
  TryLogin: "tryLogin",
  TryRegister: "tryRegister",
  Login: "login",
};

let increment = (_) => ++state.counter;
let decrement = (_) => --state.counter;
let route = (url) => {
  window.history.pushState({}, {}, url);
  state.route = url;
};
let routeBack = (url) => {
  state.route = url;
};
let tryLogin = (data) => {
  state.pendingRequests.push({
    name: Requests.Login,
    parameters: {
      email: data.get("email"),
      password: data.get("password"),
      isRemembered: data.get("rememberMe") === "on",
    },
  });
};

let tryRegister = (data) => {
  state.pendingRequests.push({
    name: Requests.Register,
    parameters: {
      email: data.get("email"),
      username: data.get("username"),
      password: data.get("password"),
    },
  });
};

let login = (email) => {
  state.login = true;
  state.email = email;
  route("/");
};

export const actionsMap = new Map([
  [Actions.Increment, increment],
  [Actions.Decrement, decrement],
  [Actions.Route, route],
  [Actions.RouteBack, routeBack],
  [Actions.TryLogin, tryLogin],
  [Actions.Login, login],
  [Actions.TryRegister, tryRegister],
]);

export function trigger(actionName, data) {
  actionsMap.get(actionName)(data);
  execute(state.pendingRequests);
  render(state);
}
window.trigger = trigger;
