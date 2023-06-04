import { trigger } from "../actions/actions";
import { ConnectToRoom } from "../actions/webrtc";
import { state } from "../state/state";
import { HttpError, HttpMethod, HttpRequest, RequestHandler } from "./fetch";

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
  state.game.channel!.send('"' + string + '"');
}

export function sendToGame(object: object) {
  state.game.channel!.send(stringifyObjectInKebabCase(object));
}

function stringifyObjectInKebabCase(obj: any): string {
  if (typeof obj !== "object" || obj === null) {
    if (typeof obj === "string") {
      return `"${obj}"`;
    }
    return String(obj);
  }

  const result: any = Array.isArray(obj) ? [] : {};

  for (const key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      const transformedKey = key.replace(
        /[A-Z]/g,
        (match) => "-" + match.toLowerCase()
      );
      result[`"${transformedKey}"`] = stringifyObjectInKebabCase(obj[key]);
    }
  }

  const keyValuePairs = Object.entries(result)
    .map(([key, value]) => `${key}:${value}`)
    .join(",");

  return `{${keyValuePairs}}`;
}
