import { PendingRequests } from '../api/fetch';
import configuration from '../configuration';

export interface State {
    route: string;
    authentication: Authentication;
    game: Game;
    pendingRequests: PendingRequests;
};

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

export let state: State = {
    route: '/',
    authentication: {
        authenticationState: AuthenticationState.LoggedOut, 
    },
    game: { isChannelOpen: false, roomId: null, channel: null, sdpOffer: null, peerConnection: configuration.rtcPeerConnection },
    pendingRequests: new PendingRequests(),
}


  