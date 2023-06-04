import { DisplaySettings, GameState, State } from "./state";

export interface DisplayState {
  route: string;
  gameState: GameState;
  displaySettings: DisplaySettings;
}

export function computeDisplayState(state: State): DisplayState {
  return {
    route: state.route,
    gameState: state.game.state,
    displaySettings: state.displaySettings,
  };
}
