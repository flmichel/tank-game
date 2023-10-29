import { DisplaySettings, GameState, State } from "./state";

export interface DisplayState {
  route: string;
  gameState: GameState;
  isInRoom: boolean;
  displaySettings: DisplaySettings;
}

export function computeDisplayState(state: State): DisplayState {
  return {
    route: state.route,
    gameState: state.game.state,
    isInRoom: state.game.roomId != null,
    displaySettings: state.displaySettings,
  };
}
