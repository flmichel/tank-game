import { state } from "../state/state";
import { view } from "../index";

export interface Action {
  execute(): void;
}

export class Reload implements Action {
  execute(): void {}
}

export class Route implements Action {
  route: string;

  constructor(route: string) {
    this.route = route;
  }

  execute(): void {
    window.history.pushState({}, "", this.route);
    state.route = this.route;
  }
}

export class RouteBack implements Action {
  previousRoute: string;

  constructor(previousRoute: string) {
    this.previousRoute = previousRoute;
  }
  execute(): void {
    state.route = this.previousRoute;
  }
}

export function trigger(action: Action) {
  action.execute();
  console.log("triggered action:", action);
  state.pendingRequests.executeAll();
  view.updateState(state);
}
