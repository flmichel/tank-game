import { Point } from "../actions/remote";
import { PendingRequests } from "../api/fetch";
import configuration from "../configuration";

export interface State {
  route: string;
  authentication: Authentication;
  game: Game;
  remote: Remote;
  pendingRequests: PendingRequests;
  displaySettings: DisplaySettings;
}

export interface Authentication {
  authenticationState: AuthenticationState;
  email?: string;
  token?: string;
}

export enum AuthenticationState {
  LoggedIn,
  LoggedOut,
  LoggingIn,
}

interface Login extends Request {
  username: string;
  password: string;
}

interface Register extends Request {
  email: string;
  username: string;
  password: string;
}

interface Game {
  isChannelOpen: boolean;
  roomId: string | null;
  channel: RTCDataChannel | null;
  sdpOffer: string | null;
  peerConnection: RTCPeerConnection;
}

interface Remote {
  leftController: LeftController;
  rightController: null;
}

interface LeftController {
  startingPoint: Point | null;
}

export interface DisplaySettings {
  windowWidth: number;
  windowHeight: number;
}

export let state: State = {
  route: "/",
  authentication: {
    authenticationState: AuthenticationState.LoggedOut,
  },
  remote: { leftController: { startingPoint: null }, rightController: null },
  game: {
    isChannelOpen: false,
    roomId: null,
    channel: null,
    sdpOffer: null,
    peerConnection: configuration.rtcPeerConnection,
  },
  displaySettings: {
    windowHeight: window.innerHeight,
    windowWidth: window.innerWidth,
  },
  pendingRequests: new PendingRequests(),
};
