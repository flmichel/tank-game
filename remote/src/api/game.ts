import { trigger } from "../actions/actions";
import { ConnectToRoom } from "../actions/webrtc";
import { state } from "../state/state";
import { HttpError, HttpMethod, HttpRequest, RequestHandler } from "./server";

export class SdpOffer implements RequestHandler {
  offer: string;
  roomId: string;

  constructor(offer: string, roomId: string) {
    this.offer = offer;
    this.roomId = roomId;
  }

  formRequest(): HttpRequest {
    return {
      method: HttpMethod.POST,
      path: "/game/" + this.roomId,
      body: this.offer,
    };
  }
  handleResponse(sdpAnswer: string): void {
    trigger(new ConnectToRoom(sdpAnswer));
  }
  handleError(error: HttpError): void {
    console.log(error);
  }
}

export function sendStringToGame(string: string) {
  state.game.channel!.send(JSON.stringify(string));
}

export function sendToGame(object: object) {
  state.game.channel!.send(stringifyObjectInKebabCase(object));
}

function stringifyObjectInKebabCase(obj: any): string {
  // The replacer renames each object's keys; JSON.stringify then recurses into the renamed copy.
  return JSON.stringify(obj, (_key, value) => {
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
      return value;
    }
    const result: any = {};
    for (const key in value) {
      if (Object.prototype.hasOwnProperty.call(value, key)) {
        result[key.replace(/[A-Z]/g, (match) => "-" + match.toLowerCase())] =
          value[key];
      }
    }
    return result;
  });
}
