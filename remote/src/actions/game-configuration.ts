import { sendStringToGame, sendToGame } from "../api/game";
import { GamePhase, state } from "../state/state";
import { Action } from "./actions";

export class UpdatePlayerName implements Action {
  playerName: string;

  constructor(playerName: string) {
    this.playerName = playerName;
  }

  execute(): void {
    state.game.state.playerName = this.playerName;
    sendToGame({ setName: this.playerName });
  }
}

export class ToggleReady implements Action {
  execute(): void {
    let isReady = !state.game.state.isReady;
    state.game.state.isReady = isReady;
    let messageToGame = isReady ? "ready" : "not-ready";
    sendStringToGame(messageToGame);

    // Since the game cannot send message back yet we consider that the game starts when this player is ready
    if (isReady) {
      state.game.state.phase = GamePhase.InGame;
    }
  }
}
