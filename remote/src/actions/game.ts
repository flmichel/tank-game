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

// Values match the game's serde serialization of MessageToRemote
export enum GameMessage {
  GameOver = "GameOver",
  GameStart = "GameStart",
  PlayerDied = "PlayerDied",
}

export class HandleGameMessage implements Action {
  message: GameMessage;

  constructor(message: GameMessage) {
    this.message = message;
  }

  execute(): void {
    if (this.message == GameMessage.GameStart) {
      state.game.state.phase = GamePhase.InGame;
      // The game resets every player to not ready when a round starts
      state.game.state.isReady = false;
    } else if (this.message == GameMessage.GameOver) {
      state.game.state.phase = GamePhase.BeforeNextGame;
    }
  }
  
}
