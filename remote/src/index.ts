import addAllEventListeners from "./event-listeners/eventListeners";
import "./view/view";
import { View } from "./view/view";

addAllEventListeners();
export const view = document.querySelector("root-view")! as View;
