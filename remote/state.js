export default {
  login: false,
  counter: 0,
  route: "/",
  email: "",
  password: "",
  loading: {
    loginIn: false,
  },
  webRTC: {
    pc: new RTCPeerConnection({
      iceServers: [
        {
          urls: "stun:stun.l.google.com:19302",
        },
      ],
    }),
  },
  pendingRequests: [],
};
