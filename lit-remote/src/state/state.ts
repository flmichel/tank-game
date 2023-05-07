export interface State {
    route: string;
    authentication: Authentication;
    game: Game;
    pendingRequests: Request[],
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

interface Request {}

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
    channel: RTCDataChannel | null;
    sdpOffer: string | null;
}

export let state: State = {
    route: '/',
    authentication: {
        authenticationState: AuthenticationState.LoggedOut, 
    },
    gameChannel: null,
    pendingRequests: [],
}


  