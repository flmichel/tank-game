import { SdpOffer } from "../api/webrtc";
import configuration from "../configuration";
import { state } from "../state/state";
import { Action, Reload, trigger } from "./actions";

export class ConfigureGameChannel implements Action {
    gameRoomId: string;

    constructor(gameRoomId: string) {
        this.gameRoomId = gameRoomId;
    }
    
    execute(): void {
        state.game.roomId = this.gameRoomId;
        let peerConnection = state.game.peerConnection;
        state.game.channel = peerConnection.createDataChannel("channel");
        console.log("channel created");
      
        peerConnection.onicecandidate = (event) => {
            console.log("onicecandidate");
            if (event.candidate && state.game.sdpOffer === null) {
                console.log("got plain offer", peerConnection.localDescription);
                let sdpOffer = btoa(JSON.stringify(peerConnection.localDescription));
                state.game.sdpOffer = sdpOffer;
                state.pendingRequests.add(new SdpOffer(sdpOffer, this.gameRoomId));
                trigger(new Reload())
            }
        }

        peerConnection.onnegotiationneeded = (e) =>
            peerConnection.createOffer().then((d) => peerConnection.setLocalDescription(d));

        console.log("peerConnection", peerConnection);
    }
}

/*export class SendSdpOffer implements Action {
    sdpOffer: string;

    constructor(sdpOffer: string) {
        this.sdpOffer = sdpOffer;
    }
    execute(): void {
        throw new Error("Method not implemented.");
    }
    
}*/

export class ConnectToRoom implements Action {
    sdpAnswer: string;

    constructor(sdpAnswer: string) {
        this.sdpAnswer = sdpAnswer;
    }
    
    execute(): void {
        state.game.peerConnection.setRemoteDescription(
            new RTCSessionDescription(JSON.parse(atob(this.sdpAnswer)))
          );
          let channel = state.game.peerConnection.createDataChannel("channel");
          channel.onclose = () => {
            state.game.isChannelOpen = false;
            console.log("channel with room has closed");
          };
          channel.onopen = () => {
            state.game.isChannelOpen = true;
            console.log("channel with room has opened");
          };
          state.game.channel = channel;
    }
}