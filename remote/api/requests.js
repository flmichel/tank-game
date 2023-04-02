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
  console.log(response);
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
  console.log("Got a response!!!");
  console.log(response);
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
  const response = await fetch("http://localhost:8000" + path, {
    method,
    mode: "cors",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  console.log(response);
  return response.json();
}
