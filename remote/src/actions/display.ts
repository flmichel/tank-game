import { state } from "../state/state";
import { Action } from "./actions";

export class SetWindowSize implements Action {
  constructor() {}
  execute(): void {
    console.log("height: ", window.innerHeight);
    console.log("width: ", window.innerWidth);
    state.displaySettings.windowHeight = window.innerHeight;
    state.displaySettings.windowWidth = window.innerWidth;
  }
}
