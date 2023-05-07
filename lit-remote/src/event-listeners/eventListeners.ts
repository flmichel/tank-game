import { ConfigureGameChannel, RouteBack, trigger } from "../actions/actions";
import configuration from "../configuration";
import { state } from "../state/state";

export default function addAllEventListeners() {
  window.onpopstate = function () {
    trigger(new RouteBack(window.location.pathname));
  };

  window.onload = function (e) {
    let urlParams = new URLSearchParams(window.location.search);
    console.log(urlParams);
    let roomId = urlParams.get("room-id");
    if (roomId !== null) {
      trigger(new ConfigureGameChannel(roomId));

      let peerConnection = configuration.rtcPeerConnection
      state.gameChannel = peerConnection.createDataChannel("channel");
      
      configuration.rtcPeerConnection.onicecandidate = (event) => {
        if (event.candidate) {
          trigger(Actions.AddSdpOffer, pc.localDescription);
        }
      };
    }

    let pc = state.webRTC.pc;

    state.webRTC.sendChannel = pc.createDataChannel("channel");

    pc.onicecandidate = (event) => {
      if (event.candidate) {
        trigger(Actions.AddSdpOffer, pc.localDescription);
      }
    };

    pc.onnegotiationneeded = (e) =>
      pc.createOffer().then((d) => pc.setLocalDescription(d));

    /*const urlParams = new URLSearchParams(window.location.search);
    console.log(urlParams);
    let roomId = urlParams.get("room-id");
    if (roomId !== null) {
      trigger(Actions.AddRoomId, roomId);
    }
    console.log("onLoad");*/
  };
}
