import { sendStringToGame, sendToGame } from "../api/game";
import { GamePhase, state } from "../state/state";
import { Action } from "./actions";
import { ConfigureGameChannel } from "./webrtc";

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

export class SignalUserId implements Action {
  execute(): void {
    let roomId = state.game.roomId!;
    let playerId = localStorage.getItem(roomId);
    if (playerId === null) {
      playerId = generateRandomId();
      localStorage.setItem(roomId, playerId);
    }
    state.game.playerId = playerId;
    sendToGame({ playerId: playerId });
  }
}

function generateRandomId(): string {
  const randomBytes = new Uint8Array(16);
  window.crypto.getRandomValues(randomBytes);
  const base64 = btoa(String.fromCharCode.apply(null, Array.from(randomBytes)))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=/g, "");

  return base64;
}

export class EnterGame implements Action {
  roomId: string;

  constructor(roomId: string) {
    this.roomId = roomId;
  }

  execute(): void {
    new ConfigureGameChannel(this.roomId).execute();
  }
}
