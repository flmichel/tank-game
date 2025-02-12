import configuration from "../configuration";

export class PendingRequests {
  requestHandlers: RequestHandler[] = [];

  add(requestHandler: RequestHandler) {
    this.requestHandlers.push(requestHandler);
  }

  async executeAll() {
    while (this.requestHandlers.length > 0) {
      this.process(this.requestHandlers.pop()!);
    }
  }

  async process(requestHandler: RequestHandler) {
    let request = requestHandler.formRequest();
    console.log("Execute request: ", request);
    const response = await fetch(configuration.apiBaseUrl + request.path, {
      method: request.method,
      mode: configuration.cors ? "cors" : "no-cors",
      body: JSON.stringify(request.body),
    });

    if (!response.ok) {
      const errorResponse = await response.json();
      const error: HttpError = {
        statusCode: response.status,
        message:
          errorResponse.message ||
          "An error occurred while processing the request.",
      };
      requestHandler.handleError(error);
    } else {
      const responseBody: SuccessfulResponse = await response.json();
      requestHandler.handleResponse(responseBody);
    }
  }
}

export interface RequestHandler {
  formRequest(): HttpRequest;
  handleResponse(response: SuccessfulResponse): void;
  handleError(error: HttpError): void;
}

export enum HttpMethod {
  GET = "get",
  POST = "post",
}

export interface HttpRequest {
  method: HttpMethod;
  path: string;
  body: HttpBody;
}

type HttpBody = Object | null;

type HttpResponse = SuccessfulResponse | HttpError;
type SuccessfulResponse = ResponseBody | string;
interface ResponseBody {}

export interface HttpError {
  statusCode: number;
  message: string;
}
