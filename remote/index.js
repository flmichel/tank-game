import { render } from "./view/view.js";
import state from "./state.js";
import addAllEventListeners from "./eventListeners.js";

addAllEventListeners();
render(state);
