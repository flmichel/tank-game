import { RouteBack, trigger } from "../actions/actions";
import { SetWindowSize } from "../actions/display";
import { ConfigureGameChannel } from "../actions/webrtc";
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
    }
  };

  window.addEventListener("resize", (event) => trigger(new SetWindowSize()));
}
