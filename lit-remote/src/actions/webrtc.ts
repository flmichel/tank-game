import configuration from "../configuration";
import { state } from "../state/state";
import { Action } from "./actions";

export class ConfigureGameChannel implements Action {
    gameRoomId: string;

    constructor(gameRoomId: string) {
        this.gameRoomId = gameRoomId;
    }
    
    execute(): void {
        let peerConnection = configuration.rtcPeerConnection
        state.game.channel = peerConnection.createDataChannel("channel");
      
        peerConnection.onicecandidate = (event) => {
            if (event.candidate && state.game.sdpOffer === null) {
                console.log("got plain offer", peerConnection.localDescription);
                let sdpOffer = btoa(JSON.stringify(peerConnection.localDescription));
                this.tryConnectToRoom(sdpOffer);
            }
        }
    }

    tryConnectToRoom(sdpOffer: string): void {
        let roomId = state.roomId;
        let sdpOffer = state.webRTC.sdpOffer;
        if (roomId !== undefined && sdpOffer !== undefined) {
          console.log("Attempting to connect to (must be done once)");
          state.pendingRequests.push({
            name: Requests.PostSdpOffer,
            parameters: {
              roomId: roomId,
              sdpOffer: sdpOffer,
            },
          });
        } else {
          console.log("Attempting to connect failed", roomId, sdpOffer);
        }
    }
}