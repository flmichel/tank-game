import { Actions, trigger } from "../actions.js";

export default function addAllEventListeners() {
  window.onpopstate = function (e) {
    trigger(Actions.RouteBack, e.currentTarget.location.pathname);
  };
}
