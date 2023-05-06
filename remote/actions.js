import { Requests } from "./api/requests.js";
import state from "./state.js";
import { render } from "./view/view.js";
import { execute } from "./api/requests.js";

export const Actions = {
  Increment: "increment",
  Decrement: "decrement",
  Route: "route",
  RouteBack: "routeBack",
  TryLogin: "tryLogin",
  TryRegister: "tryRegister",
  Login: "login",
  AddSdpOffer: "addSdpOffer",
  AddRoomId: "addRoomId",
  ConnectToRoom: "connectToRoom",
};

let increment = (_) => {
  ++state.counter;
  console.log("A");
  if (state.webRTC.isChannelOpen) {
    state.webRTC.roomChannel.send(JSON.stringify("A"));
  }
};
let decrement = (_) => {
  --state.counter;
  if (state.webRTC.isChannelOpen) {
    state.webRTC.roomChannel.send(JSON.stringify("B"));
  }
};
let route = (url) => {
  window.history.pushState({}, {}, url);
  state.route = url;
};
let routeBack = (url) => {
  state.route = url;
};
let tryLogin = (data) => {
  state.pendingRequests.push({
    name: Requests.Login,
    parameters: {
      email: data.get("email"),
      password: data.get("password"),
      isRemembered: data.get("rememberMe") === "on",
    },
  });
};

let tryRegister = (data) => {
  state.pendingRequests.push({
    name: Requests.Register,
    parameters: {
      email: data.get("email"),
      username: data.get("username"),
      password: data.get("password"),
    },
  });
};

let login = (email) => {
  state.login = true;
  state.email = email;
  route("/");
};

let tryConnectToRoom = () => {
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
};

let addSdpOffer = (plainOffer) => {
  if (state.webRTC.sdpOffer === undefined) {
    console.log("got plain offer", plainOffer);
    state.webRTC.sdpOffer = btoa(JSON.stringify(plainOffer));
    tryConnectToRoom();
  }
};

let addRoomId = (roomId) => {
  if (state.roomId === undefined) {
    console.log("go once room id");
    state.roomId = roomId;
    tryConnectToRoom();
  }
};

let connectToRoom = (sdpAnswer) => {
  state.webRTC.pc.setRemoteDescription(
    new RTCSessionDescription(JSON.parse(atob(sdpAnswer)))
  );
  let roomChannel = state.webRTC.pc.createDataChannel("channel");
  roomChannel.onclose = () => {
    state.webRTC.isChannelOpen = false;
    console.log("channel with room has closed");
  };
  roomChannel.onopen = () => {
    state.webRTC.isChannelOpen = true;
    console.log("channel with room has opened");
  };
  state.webRTC.roomChannel = roomChannel;
};

export const actionsMap = new Map([
  [Actions.Increment, increment],
  [Actions.Decrement, decrement],
  [Actions.Route, route],
  [Actions.RouteBack, routeBack],
  [Actions.TryLogin, tryLogin],
  [Actions.Login, login],
  [Actions.TryRegister, tryRegister],
  [Actions.AddSdpOffer, addSdpOffer],
  [Actions.AddRoomId, addRoomId],
  [Actions.ConnectToRoom, connectToRoom],
]);

export function trigger(actionName, data) {
  console.log("triggerd action:", actionName);
  actionsMap.get(actionName)(data);
  console.log(state);
  execute(state.pendingRequests);
  render(state);
}
window.trigger = trigger;
