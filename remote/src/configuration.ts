export default {
  rtcPeerConnection: new RTCPeerConnection({
    iceServers: [
      {
        urls: ["stun:stun.l.google.com:19302", "stun:stun3.l.google.com:19302"],
      },
    ],
  }),
};

export const apiBaseUrl =
  process.env.API_BASE_URL ?? "https://tank-game.flmichel.duckdns.org/api";
export const cors = process.env.CORS === "true" ? "cors" : "no-cors";
export const NUMBER_OF_ICE_CANDIDATES = parseInt(
  process.env.NUMBER_OF_ICE_CANDIDATES ?? "8"
);
