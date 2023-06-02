import { trigger } from "../actions/actions";
import { ConnectToRoom } from "../actions/webrtc";
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
        }
    }
    handleResponse(sdpAnswer: string): void {
        trigger(new ConnectToRoom(sdpAnswer))
    }
    handleError(error: HttpError): void {
        console.log(error)
    }
}
