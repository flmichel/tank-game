import { state } from "../state/state";
import { Action } from "./actions";

export class SetWindowSize implements Action {
  constructor() {}
  execute(): void {
    state.displaySettings.windowHeight = window.innerHeight;
    state.displaySettings.windowWidth = window.innerWidth;
  }
}
