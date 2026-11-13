import SuperTokens from "supertokens-web-js";
import Session from "supertokens-web-js/recipe/session";
import ThirdParty from "supertokens-web-js/recipe/thirdparty";
import { AUTH_MODE } from "#/lib/auth-mode";

let initialized = false;

/** Initializes the browser SDK on the same origin as the authentication routes. */
export function initAuth(): void {
	if (initialized || AUTH_MODE === "off" || typeof window === "undefined")
		return;
	SuperTokens.init({
		appInfo: {
			appName: "Grocery List",
			apiDomain: window.location.origin,
			apiBasePath: "/api/auth",
		},
		recipeList: [
			ThirdParty.init(),
			Session.init({ tokenTransferMethod: "header" }),
		],
	});
	initialized = true;
}

/** Reads the session token, refreshing an expired session through the SDK. */
export async function getAuthToken(): Promise<string | null> {
	initAuth();
	if (!initialized || !(await Session.doesSessionExist())) return null;
	return (await Session.getAccessToken()) ?? null;
}

/** Asks the SDK to refresh a session rejected by the backend. */
export async function refreshAuth(): Promise<boolean> {
	return initialized && Session.attemptRefreshingSession();
}
