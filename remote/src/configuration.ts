export default {
  rtcPeerConnection: new RTCPeerConnection({
    iceServers: [
      {
        urls: "stun:stun.l.google.com:19302",
      },
    ],
  }),
};

export const apiBaseUrl = process.env.API_BASE_URL;
export const cors = process.env.CORS === "true" ? "cors" : "no-cors";
