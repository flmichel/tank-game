import { Actions, trigger } from "./actions.js";
import state from "./state.js";

export default function addAllEventListeners() {
  window.onpopstate = function (e) {
    trigger(Actions.RouteBack, e.currentTarget.location.pathname);
  };

  window.onload = function (e) {
    let pc = state.webRTC.pc;

    state.webRTC.sendChannel = pc.createDataChannel("channel");

    pc.onicecandidate = (event) => {
      if (event.candidate) {
        trigger(Actions.AddSdpOffer, pc.localDescription);
      }
    };

    pc.onnegotiationneeded = (e) =>
      pc.createOffer().then((d) => pc.setLocalDescription(d));

    const urlParams = new URLSearchParams(window.location.search);
    console.log(urlParams);
    let roomId = urlParams.get("room-id");
    if (roomId !== null) {
      trigger(Actions.AddRoomId, roomId);
    }
    console.log("onLoad");
  };

  /*window.onload = function (e) {
    trigger(Actions.RequestSdpOffer);
  };*/

  state.webRTC.pc.onnegotiationneeded = (e) => {
    pc.createOffer()
      .then((d) => pc.setLocalDescription(d))
      .catch(log);
  };

  state.webRTC.pc.onicecandidate = function (e) {
    console.log(pc.localDescription);
    btoa(JSON.stringify(pc.localDescription));
  };
}
