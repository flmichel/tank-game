/*
 * ATTENTION: An "eval-source-map" devtool has been used.
 * This devtool is neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file with attached SourceMaps in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
/******/ (() => { // webpackBootstrap
/******/ 	var __webpack_modules__ = ({

/***/ "./src/actions/actions.ts":
/*!********************************!*\
  !*** ./src/actions/actions.ts ***!
  \********************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"Route\": () => (/* binding */ Route),\n/* harmony export */   \"RouteBack\": () => (/* binding */ RouteBack),\n/* harmony export */   \"trigger\": () => (/* binding */ trigger)\n/* harmony export */ });\n/* harmony import */ var _state_state__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ../state/state */ \"./src/state/state.ts\");\n/* harmony import */ var _index__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../index */ \"./src/index.ts\");\n\n\nclass Route {\n    constructor(route) {\n        this.route = route;\n    }\n    execute() {\n        window.history.pushState({}, '', this.route);\n        _state_state__WEBPACK_IMPORTED_MODULE_0__.state.route = this.route;\n    }\n}\nclass RouteBack {\n    constructor(previousRoute) {\n        this.previousRoute = previousRoute;\n    }\n    execute() {\n        _state_state__WEBPACK_IMPORTED_MODULE_0__.state.route = this.previousRoute;\n    }\n}\nfunction trigger(action) {\n    action.execute();\n    console.log(\"triggered action:\", action);\n    _index__WEBPACK_IMPORTED_MODULE_1__.view.state = _state_state__WEBPACK_IMPORTED_MODULE_0__.state;\n    _index__WEBPACK_IMPORTED_MODULE_1__.view.requestUpdate();\n}\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi9zcmMvYWN0aW9ucy9hY3Rpb25zLnRzLmpzIiwibWFwcGluZ3MiOiI7Ozs7Ozs7O0FBQXVDO0FBQ1A7QUFPekIsTUFBTSxLQUFLO0lBR2QsWUFBWSxLQUFhO1FBQ3JCLElBQUksQ0FBQyxLQUFLLEdBQUcsS0FBSyxDQUFDO0lBQ3ZCLENBQUM7SUFFRCxPQUFPO1FBQ0gsTUFBTSxDQUFDLE9BQU8sQ0FBQyxTQUFTLENBQUMsRUFBRSxFQUFFLEVBQUUsRUFBRSxJQUFJLENBQUMsS0FBSyxDQUFDLENBQUM7UUFDN0MscURBQVcsR0FBRyxJQUFJLENBQUMsS0FBSyxDQUFDO0lBQzdCLENBQUM7Q0FDSjtBQUVNLE1BQU0sU0FBUztJQUdsQixZQUFZLGFBQXFCO1FBQzdCLElBQUksQ0FBQyxhQUFhLEdBQUcsYUFBYSxDQUFDO0lBQ3ZDLENBQUM7SUFFRCxPQUFPO1FBQ0gscURBQVcsR0FBRyxJQUFJLENBQUMsYUFBYSxDQUFDO0lBQ3JDLENBQUM7Q0FDSjtBQUVNLFNBQVMsT0FBTyxDQUFDLE1BQWM7SUFDbEMsTUFBTSxDQUFDLE9BQU8sRUFBRSxDQUFDO0lBQ2pCLE9BQU8sQ0FBQyxHQUFHLENBQUMsbUJBQW1CLEVBQUUsTUFBTSxDQUFDLENBQUM7SUFDekMsOENBQVUsR0FBRywrQ0FBSyxDQUFDO0lBQ25CLHNEQUFrQixFQUFFLENBQUM7QUFDekIsQ0FBQyIsInNvdXJjZXMiOlsid2VicGFjazovL2xpdC1yZW1vdGUvLi9zcmMvYWN0aW9ucy9hY3Rpb25zLnRzP2YzM2YiXSwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgc3RhdGUgfSBmcm9tIFwiLi4vc3RhdGUvc3RhdGVcIjtcbmltcG9ydCB7IHZpZXcgfSBmcm9tIFwiLi4vaW5kZXhcIjtcbmltcG9ydCBjb25maWd1cmF0aW9uIGZyb20gXCIuLi9jb25maWd1cmF0aW9uXCI7XG5cbmV4cG9ydCBpbnRlcmZhY2UgQWN0aW9uIHtcbiAgICBleGVjdXRlKCk6IHZvaWQ7XG59XG5cbmV4cG9ydCBjbGFzcyBSb3V0ZSBpbXBsZW1lbnRzIEFjdGlvbiB7XG4gICAgcm91dGU6IHN0cmluZztcblxuICAgIGNvbnN0cnVjdG9yKHJvdXRlOiBzdHJpbmcpIHtcbiAgICAgICAgdGhpcy5yb3V0ZSA9IHJvdXRlO1xuICAgIH1cbiAgICBcbiAgICBleGVjdXRlKCk6IHZvaWQge1xuICAgICAgICB3aW5kb3cuaGlzdG9yeS5wdXNoU3RhdGUoe30sICcnLCB0aGlzLnJvdXRlKTtcbiAgICAgICAgc3RhdGUucm91dGUgPSB0aGlzLnJvdXRlO1xuICAgIH1cbn1cblxuZXhwb3J0IGNsYXNzIFJvdXRlQmFjayBpbXBsZW1lbnRzIEFjdGlvbiB7XG4gICAgcHJldmlvdXNSb3V0ZTogc3RyaW5nO1xuXG4gICAgY29uc3RydWN0b3IocHJldmlvdXNSb3V0ZTogc3RyaW5nKSB7XG4gICAgICAgIHRoaXMucHJldmlvdXNSb3V0ZSA9IHByZXZpb3VzUm91dGU7XG4gICAgfVxuICAgIFxuICAgIGV4ZWN1dGUoKTogdm9pZCB7XG4gICAgICAgIHN0YXRlLnJvdXRlID0gdGhpcy5wcmV2aW91c1JvdXRlO1xuICAgIH1cbn1cblxuZXhwb3J0IGZ1bmN0aW9uIHRyaWdnZXIoYWN0aW9uOiBBY3Rpb24pIHtcbiAgICBhY3Rpb24uZXhlY3V0ZSgpO1xuICAgIGNvbnNvbGUubG9nKFwidHJpZ2dlcmVkIGFjdGlvbjpcIiwgYWN0aW9uKTtcbiAgICB2aWV3LnN0YXRlID0gc3RhdGU7XG4gICAgdmlldy5yZXF1ZXN0VXBkYXRlKCk7XG59Il0sIm5hbWVzIjpbXSwic291cmNlUm9vdCI6IiJ9\n//# sourceURL=webpack-internal:///./src/actions/actions.ts\n");

/***/ }),

/***/ "./src/configuration.ts":
/*!******************************!*\
  !*** ./src/configuration.ts ***!
  \******************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = ({\n    rtcPeerConnection: new RTCPeerConnection({\n        iceServers: [\n            {\n                urls: \"stun:stun.l.google.com:19302\",\n            },\n        ],\n    }),\n});\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi9zcmMvY29uZmlndXJhdGlvbi50cy5qcyIsIm1hcHBpbmdzIjoiOzs7O0FBQUEsaUVBQWU7SUFDYixpQkFBaUIsRUFBRSxJQUFJLGlCQUFpQixDQUFDO1FBQ3ZDLFVBQVUsRUFBRTtZQUNWO2dCQUNFLElBQUksRUFBRSw4QkFBOEI7YUFDckM7U0FDRjtLQUNGLENBQUM7Q0FDSCIsInNvdXJjZXMiOlsid2VicGFjazovL2xpdC1yZW1vdGUvLi9zcmMvY29uZmlndXJhdGlvbi50cz8zZDI5Il0sInNvdXJjZXNDb250ZW50IjpbImV4cG9ydCBkZWZhdWx0IHtcbiAgcnRjUGVlckNvbm5lY3Rpb246IG5ldyBSVENQZWVyQ29ubmVjdGlvbih7XG4gICAgaWNlU2VydmVyczogW1xuICAgICAge1xuICAgICAgICB1cmxzOiBcInN0dW46c3R1bi5sLmdvb2dsZS5jb206MTkzMDJcIixcbiAgICAgIH0sXG4gICAgXSxcbiAgfSksXG59Il0sIm5hbWVzIjpbXSwic291cmNlUm9vdCI6IiJ9\n//# sourceURL=webpack-internal:///./src/configuration.ts\n");

/***/ }),

/***/ "./src/event-listeners/eventListeners.ts":
/*!***********************************************!*\
  !*** ./src/event-listeners/eventListeners.ts ***!
  \***********************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (/* binding */ addAllEventListeners)\n/* harmony export */ });\n/* harmony import */ var _actions_actions__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ../actions/actions */ \"./src/actions/actions.ts\");\n/* harmony import */ var _configuration__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../configuration */ \"./src/configuration.ts\");\n/* harmony import */ var _state_state__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ../state/state */ \"./src/state/state.ts\");\n\n\n\nfunction addAllEventListeners() {\n    window.onpopstate = function () {\n        (0,_actions_actions__WEBPACK_IMPORTED_MODULE_0__.trigger)(new _actions_actions__WEBPACK_IMPORTED_MODULE_0__.RouteBack(window.location.pathname));\n    };\n    window.onload = function (e) {\n        let urlParams = new URLSearchParams(window.location.search);\n        console.log(urlParams);\n        let roomId = urlParams.get(\"room-id\");\n        if (roomId !== null) {\n            (0,_actions_actions__WEBPACK_IMPORTED_MODULE_0__.trigger)(new _actions_actions__WEBPACK_IMPORTED_MODULE_0__.ConfigureGameChannel(roomId));\n            let peerConnection = _configuration__WEBPACK_IMPORTED_MODULE_1__[\"default\"].rtcPeerConnection;\n            _state_state__WEBPACK_IMPORTED_MODULE_2__.state.gameChannel = peerConnection.createDataChannel(\"channel\");\n            _configuration__WEBPACK_IMPORTED_MODULE_1__[\"default\"].rtcPeerConnection.onicecandidate = (event) => {\n                if (event.candidate) {\n                    (0,_actions_actions__WEBPACK_IMPORTED_MODULE_0__.trigger)(Actions.AddSdpOffer, pc.localDescription);\n                }\n            };\n        }\n        let pc = _state_state__WEBPACK_IMPORTED_MODULE_2__.state.webRTC.pc;\n        _state_state__WEBPACK_IMPORTED_MODULE_2__.state.webRTC.sendChannel = pc.createDataChannel(\"channel\");\n        pc.onicecandidate = (event) => {\n            if (event.candidate) {\n                (0,_actions_actions__WEBPACK_IMPORTED_MODULE_0__.trigger)(Actions.AddSdpOffer, pc.localDescription);\n            }\n        };\n        pc.onnegotiationneeded = (e) => pc.createOffer().then((d) => pc.setLocalDescription(d));\n        /*const urlParams = new URLSearchParams(window.location.search);\n        console.log(urlParams);\n        let roomId = urlParams.get(\"room-id\");\n        if (roomId !== null) {\n          trigger(Actions.AddRoomId, roomId);\n        }\n        console.log(\"onLoad\");*/\n    };\n}\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi9zcmMvZXZlbnQtbGlzdGVuZXJzL2V2ZW50TGlzdGVuZXJzLnRzLmpzIiwibWFwcGluZ3MiOiI7Ozs7Ozs7QUFBOEU7QUFDakM7QUFDTjtBQUV4QixTQUFTLG9CQUFvQjtJQUMxQyxNQUFNLENBQUMsVUFBVSxHQUFHO1FBQ2xCLHlEQUFPLENBQUMsSUFBSSx1REFBUyxDQUFDLE1BQU0sQ0FBQyxRQUFRLENBQUMsUUFBUSxDQUFDLENBQUMsQ0FBQztJQUNuRCxDQUFDLENBQUM7SUFFRixNQUFNLENBQUMsTUFBTSxHQUFHLFVBQVUsQ0FBQztRQUN6QixJQUFJLFNBQVMsR0FBRyxJQUFJLGVBQWUsQ0FBQyxNQUFNLENBQUMsUUFBUSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQzVELE9BQU8sQ0FBQyxHQUFHLENBQUMsU0FBUyxDQUFDLENBQUM7UUFDdkIsSUFBSSxNQUFNLEdBQUcsU0FBUyxDQUFDLEdBQUcsQ0FBQyxTQUFTLENBQUMsQ0FBQztRQUN0QyxJQUFJLE1BQU0sS0FBSyxJQUFJLEVBQUU7WUFDbkIseURBQU8sQ0FBQyxJQUFJLGtFQUFvQixDQUFDLE1BQU0sQ0FBQyxDQUFDLENBQUM7WUFFMUMsSUFBSSxjQUFjLEdBQUcsd0VBQStCO1lBQ3BELDJEQUFpQixHQUFHLGNBQWMsQ0FBQyxpQkFBaUIsQ0FBQyxTQUFTLENBQUMsQ0FBQztZQUVoRSx1RkFBOEMsR0FBRyxDQUFDLEtBQUssRUFBRSxFQUFFO2dCQUN6RCxJQUFJLEtBQUssQ0FBQyxTQUFTLEVBQUU7b0JBQ25CLHlEQUFPLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxFQUFFLENBQUMsZ0JBQWdCLENBQUMsQ0FBQztpQkFDbkQ7WUFDSCxDQUFDLENBQUM7U0FDSDtRQUVELElBQUksRUFBRSxHQUFHLHlEQUFlLENBQUM7UUFFekIsa0VBQXdCLEdBQUcsRUFBRSxDQUFDLGlCQUFpQixDQUFDLFNBQVMsQ0FBQyxDQUFDO1FBRTNELEVBQUUsQ0FBQyxjQUFjLEdBQUcsQ0FBQyxLQUFLLEVBQUUsRUFBRTtZQUM1QixJQUFJLEtBQUssQ0FBQyxTQUFTLEVBQUU7Z0JBQ25CLHlEQUFPLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxFQUFFLENBQUMsZ0JBQWdCLENBQUMsQ0FBQzthQUNuRDtRQUNILENBQUMsQ0FBQztRQUVGLEVBQUUsQ0FBQyxtQkFBbUIsR0FBRyxDQUFDLENBQUMsRUFBRSxFQUFFLENBQzdCLEVBQUUsQ0FBQyxXQUFXLEVBQUUsQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDLEVBQUUsRUFBRSxDQUFDLEVBQUUsQ0FBQyxtQkFBbUIsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDO1FBRTFEOzs7Ozs7Z0NBTXdCO0lBQzFCLENBQUMsQ0FBQztBQUNKLENBQUMiLCJzb3VyY2VzIjpbIndlYnBhY2s6Ly9saXQtcmVtb3RlLy4vc3JjL2V2ZW50LWxpc3RlbmVycy9ldmVudExpc3RlbmVycy50cz84NDg4Il0sInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IENvbmZpZ3VyZUdhbWVDaGFubmVsLCBSb3V0ZUJhY2ssIHRyaWdnZXIgfSBmcm9tIFwiLi4vYWN0aW9ucy9hY3Rpb25zXCI7XG5pbXBvcnQgY29uZmlndXJhdGlvbiBmcm9tIFwiLi4vY29uZmlndXJhdGlvblwiO1xuaW1wb3J0IHsgc3RhdGUgfSBmcm9tIFwiLi4vc3RhdGUvc3RhdGVcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24gYWRkQWxsRXZlbnRMaXN0ZW5lcnMoKSB7XG4gIHdpbmRvdy5vbnBvcHN0YXRlID0gZnVuY3Rpb24gKCkge1xuICAgIHRyaWdnZXIobmV3IFJvdXRlQmFjayh3aW5kb3cubG9jYXRpb24ucGF0aG5hbWUpKTtcbiAgfTtcblxuICB3aW5kb3cub25sb2FkID0gZnVuY3Rpb24gKGUpIHtcbiAgICBsZXQgdXJsUGFyYW1zID0gbmV3IFVSTFNlYXJjaFBhcmFtcyh3aW5kb3cubG9jYXRpb24uc2VhcmNoKTtcbiAgICBjb25zb2xlLmxvZyh1cmxQYXJhbXMpO1xuICAgIGxldCByb29tSWQgPSB1cmxQYXJhbXMuZ2V0KFwicm9vbS1pZFwiKTtcbiAgICBpZiAocm9vbUlkICE9PSBudWxsKSB7XG4gICAgICB0cmlnZ2VyKG5ldyBDb25maWd1cmVHYW1lQ2hhbm5lbChyb29tSWQpKTtcblxuICAgICAgbGV0IHBlZXJDb25uZWN0aW9uID0gY29uZmlndXJhdGlvbi5ydGNQZWVyQ29ubmVjdGlvblxuICAgICAgc3RhdGUuZ2FtZUNoYW5uZWwgPSBwZWVyQ29ubmVjdGlvbi5jcmVhdGVEYXRhQ2hhbm5lbChcImNoYW5uZWxcIik7XG4gICAgICBcbiAgICAgIGNvbmZpZ3VyYXRpb24ucnRjUGVlckNvbm5lY3Rpb24ub25pY2VjYW5kaWRhdGUgPSAoZXZlbnQpID0+IHtcbiAgICAgICAgaWYgKGV2ZW50LmNhbmRpZGF0ZSkge1xuICAgICAgICAgIHRyaWdnZXIoQWN0aW9ucy5BZGRTZHBPZmZlciwgcGMubG9jYWxEZXNjcmlwdGlvbik7XG4gICAgICAgIH1cbiAgICAgIH07XG4gICAgfVxuXG4gICAgbGV0IHBjID0gc3RhdGUud2ViUlRDLnBjO1xuXG4gICAgc3RhdGUud2ViUlRDLnNlbmRDaGFubmVsID0gcGMuY3JlYXRlRGF0YUNoYW5uZWwoXCJjaGFubmVsXCIpO1xuXG4gICAgcGMub25pY2VjYW5kaWRhdGUgPSAoZXZlbnQpID0+IHtcbiAgICAgIGlmIChldmVudC5jYW5kaWRhdGUpIHtcbiAgICAgICAgdHJpZ2dlcihBY3Rpb25zLkFkZFNkcE9mZmVyLCBwYy5sb2NhbERlc2NyaXB0aW9uKTtcbiAgICAgIH1cbiAgICB9O1xuXG4gICAgcGMub25uZWdvdGlhdGlvbm5lZWRlZCA9IChlKSA9PlxuICAgICAgcGMuY3JlYXRlT2ZmZXIoKS50aGVuKChkKSA9PiBwYy5zZXRMb2NhbERlc2NyaXB0aW9uKGQpKTtcblxuICAgIC8qY29uc3QgdXJsUGFyYW1zID0gbmV3IFVSTFNlYXJjaFBhcmFtcyh3aW5kb3cubG9jYXRpb24uc2VhcmNoKTtcbiAgICBjb25zb2xlLmxvZyh1cmxQYXJhbXMpO1xuICAgIGxldCByb29tSWQgPSB1cmxQYXJhbXMuZ2V0KFwicm9vbS1pZFwiKTtcbiAgICBpZiAocm9vbUlkICE9PSBudWxsKSB7XG4gICAgICB0cmlnZ2VyKEFjdGlvbnMuQWRkUm9vbUlkLCByb29tSWQpO1xuICAgIH1cbiAgICBjb25zb2xlLmxvZyhcIm9uTG9hZFwiKTsqL1xuICB9O1xufVxuIl0sIm5hbWVzIjpbXSwic291cmNlUm9vdCI6IiJ9\n//# sourceURL=webpack-internal:///./src/event-listeners/eventListeners.ts\n");

/***/ }),

/***/ "./src/index.ts":
/*!**********************!*\
  !*** ./src/index.ts ***!
  \**********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"view\": () => (/* binding */ view)\n/* harmony export */ });\n/* harmony import */ var _event_listeners_eventListeners__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./event-listeners/eventListeners */ \"./src/event-listeners/eventListeners.ts\");\n/* harmony import */ var _view_view__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./view/view */ \"./src/view/view.ts\");\n\n\n(0,_event_listeners_eventListeners__WEBPACK_IMPORTED_MODULE_0__[\"default\"])();\nconst view = document.querySelector('root-view');\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi9zcmMvaW5kZXgudHMuanMiLCJtYXBwaW5ncyI6Ijs7Ozs7O0FBQW9FO0FBQy9DO0FBR3JCLDJFQUFvQixFQUFFLENBQUM7QUFDaEIsTUFBTSxJQUFJLEdBQUcsUUFBUSxDQUFDLGFBQWEsQ0FBQyxXQUFXLENBQVUsQ0FBQyIsInNvdXJjZXMiOlsid2VicGFjazovL2xpdC1yZW1vdGUvLi9zcmMvaW5kZXgudHM/ZmZiNCJdLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgYWRkQWxsRXZlbnRMaXN0ZW5lcnMgZnJvbSAnLi9ldmVudC1saXN0ZW5lcnMvZXZlbnRMaXN0ZW5lcnMnO1xuaW1wb3J0ICcuL3ZpZXcvdmlldyc7XG5pbXBvcnQgeyBWaWV3IH0gZnJvbSAnLi92aWV3L3ZpZXcnO1xuXG5hZGRBbGxFdmVudExpc3RlbmVycygpO1xuZXhwb3J0IGNvbnN0IHZpZXcgPSBkb2N1bWVudC5xdWVyeVNlbGVjdG9yKCdyb290LXZpZXcnKSEgYXMgVmlldztcbiJdLCJuYW1lcyI6W10sInNvdXJjZVJvb3QiOiIifQ==\n//# sourceURL=webpack-internal:///./src/index.ts\n");

/***/ }),

/***/ "./src/state/state.ts":
/*!****************************!*\
  !*** ./src/state/state.ts ***!
  \****************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"AuthenticationState\": () => (/* binding */ AuthenticationState),\n/* harmony export */   \"state\": () => (/* binding */ state)\n/* harmony export */ });\n;\nvar AuthenticationState;\n(function (AuthenticationState) {\n    AuthenticationState[AuthenticationState[\"LoggedIn\"] = 0] = \"LoggedIn\";\n    AuthenticationState[AuthenticationState[\"LoggedOut\"] = 1] = \"LoggedOut\";\n    AuthenticationState[AuthenticationState[\"LoggingIn\"] = 2] = \"LoggingIn\";\n})(AuthenticationState || (AuthenticationState = {}));\nlet state = {\n    route: '/',\n    authentication: {\n        authenticationState: AuthenticationState.LoggedOut,\n    },\n    gameChannel: null,\n    pendingRequests: [],\n};\n//# sourceURL=[module]\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiLi9zcmMvc3RhdGUvc3RhdGUudHMuanMiLCJtYXBwaW5ncyI6Ijs7Ozs7QUFLQyxDQUFDO0FBUUYsSUFBWSxtQkFJWDtBQUpELFdBQVksbUJBQW1CO0lBQzNCLHFFQUFRO0lBQ1IsdUVBQVM7SUFDVCx1RUFBUztBQUNiLENBQUMsRUFKVyxtQkFBbUIsS0FBbkIsbUJBQW1CLFFBSTlCO0FBb0JNLElBQUksS0FBSyxHQUFVO0lBQ3RCLEtBQUssRUFBRSxHQUFHO0lBQ1YsY0FBYyxFQUFFO1FBQ1osbUJBQW1CLEVBQUUsbUJBQW1CLENBQUMsU0FBUztLQUNyRDtJQUNELFdBQVcsRUFBRSxJQUFJO0lBQ2pCLGVBQWUsRUFBRSxFQUFFO0NBQ3RCIiwic291cmNlcyI6WyJ3ZWJwYWNrOi8vbGl0LXJlbW90ZS8uL3NyYy9zdGF0ZS9zdGF0ZS50cz9iYmIwIl0sInNvdXJjZXNDb250ZW50IjpbImV4cG9ydCBpbnRlcmZhY2UgU3RhdGUge1xuICAgIHJvdXRlOiBzdHJpbmc7XG4gICAgYXV0aGVudGljYXRpb246IEF1dGhlbnRpY2F0aW9uO1xuICAgIGdhbWU6IEdhbWU7XG4gICAgcGVuZGluZ1JlcXVlc3RzOiBSZXF1ZXN0W10sXG59O1xuXG5leHBvcnQgaW50ZXJmYWNlIEF1dGhlbnRpY2F0aW9uIHtcbiAgICBhdXRoZW50aWNhdGlvblN0YXRlOiBBdXRoZW50aWNhdGlvblN0YXRlO1xuICAgIGVtYWlsPzogc3RyaW5nO1xuICAgIHRva2VuPzogc3RyaW5nO1xufVxuXG5leHBvcnQgZW51bSBBdXRoZW50aWNhdGlvblN0YXRlIHtcbiAgICBMb2dnZWRJbixcbiAgICBMb2dnZWRPdXQsXG4gICAgTG9nZ2luZ0luLFxufVxuXG5pbnRlcmZhY2UgUmVxdWVzdCB7fVxuXG5pbnRlcmZhY2UgTG9naW4gZXh0ZW5kcyBSZXF1ZXN0IHtcbiAgICB1c2VybmFtZTogc3RyaW5nO1xuICAgIHBhc3N3b3JkOiBzdHJpbmc7XG59XG5cbmludGVyZmFjZSBSZWdpc3RlciBleHRlbmRzIFJlcXVlc3Qge1xuICAgIGVtYWlsOiBzdHJpbmc7XG4gICAgdXNlcm5hbWU6IHN0cmluZztcbiAgICBwYXNzd29yZDogc3RyaW5nO1xufVxuXG5pbnRlcmZhY2UgR2FtZSB7XG4gICAgY2hhbm5lbDogUlRDRGF0YUNoYW5uZWwgfCBudWxsO1xuICAgIHNkcE9mZmVyOiBzdHJpbmcgfCBudWxsO1xufVxuXG5leHBvcnQgbGV0IHN0YXRlOiBTdGF0ZSA9IHtcbiAgICByb3V0ZTogJy8nLFxuICAgIGF1dGhlbnRpY2F0aW9uOiB7XG4gICAgICAgIGF1dGhlbnRpY2F0aW9uU3RhdGU6IEF1dGhlbnRpY2F0aW9uU3RhdGUuTG9nZ2VkT3V0LCBcbiAgICB9LFxuICAgIGdhbWVDaGFubmVsOiBudWxsLFxuICAgIHBlbmRpbmdSZXF1ZXN0czogW10sXG59XG5cblxuICAiXSwibmFtZXMiOltdLCJzb3VyY2VSb290IjoiIn0=\n//# sourceURL=webpack-internal:///./src/state/state.ts\n");

/***/ }),

/***/ "./src/view/view.ts":
/*!**************************!*\
  !*** ./src/view/view.ts ***!
  \**************************/
/***/ (() => {

throw new Error("Module parse failed: Unterminated regular expression (29:30)\nFile was processed with these loaders:\n * ./node_modules/ts-loader/index.js\nYou may need an additional loader to handle the result of these loaders.\n|     renderPage(route) {\n|         switch (route) {\n>             case '/': return /homepage>;;\n|         }\n|     }");

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// startup
/******/ 	// Load entry module and return exports
/******/ 	// This entry module is referenced by other modules so it can't be inlined
/******/ 	var __webpack_exports__ = __webpack_require__("./src/index.ts");
/******/ 	
/******/ })()
;