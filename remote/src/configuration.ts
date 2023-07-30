export default {
  rtcPeerConnection: new RTCPeerConnection({
    iceServers: [
      {
        urls: [
          "stun:stun.l.google.com:19302",
          "stun:stun3.l.google.com:19302",
          "stun:stun.services.mozilla.com",
          "stun:stun.demo.cloud.webrtc.org",
        ],
      },
    ],
  }),
  apiBaseUrl:
    process.env.API_BASE_URL ?? "https://tank-game.flmichel.duckdns.org/api",
  cors: process.env.CORS === "true" ? true : false,
};
