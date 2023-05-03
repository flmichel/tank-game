import { trigger, Actions } from "../actions.js";

export const Requests = {
  Login: "login",
  Register: "register",
  PostSdpOffer: "postSdpOffer",
};

async function login(parameters) {
  console.log(parameters.email);
  const response = await post("/login", {
    email: parameters.email,
    password: parameters.password,
  });
  return [Actions.Login, parameters.email];
}

async function register(parameters) {
  const response = await post("/user/registration", {
    email: parameters.email,
    name: parameters.username,
    password: parameters.password,
  });
  console.log(response);
  return [Actions.Login, parameters.email];
}

async function postSdpOffer(parameters) {
  const response = await post(
    "/game/" + parameters.roomId,
    parameters.sdpOffer
  );
  console.log("sdp answer", response);
  return [Actions.ConnectToRoom, response];
}

export const requestsMap = new Map([
  [Requests.Login, login],
  [Requests.Register, register],
  [Requests.PostSdpOffer, postSdpOffer],
]);

export async function execute(pendingRequests) {
  pendingRequests.forEach((request) => {
    pendingRequests.pop(request);
    console.log(request);
    requestsMap
      .get(request.name)(request.parameters)
      .then(([action, data]) => {
        console.log(action);
        console.log(data);
        trigger(action, data);
      });
  });
}

async function post(path, data = {}) {
  return request("POST", path, data);
}

async function get(path, data = {}) {
  return request("GET", path, data);
}

async function request(method, path, data) {
  console.log(JSON.stringify(data));

  let headers = new Headers();

  headers.append("Content-Type", "application/json");

  const response = await fetch(
    "https://8c83-188-61-172-167.eu.ngrok.io" + path,
    //"http://localhost:8000" + path,
    {
      method,
      mode: "cors",
      headers: headers,
      body: JSON.stringify(data),
    }
  );

  return response.json();
}
