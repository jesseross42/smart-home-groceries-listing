import type { AuthMode } from "#/lib/auth-mode";

/** The household sign-in page. */
export const SIGN_IN_PATH = "/sign-in";

/** Whether this location belongs to sign-in, including its OIDC callback. */
export function isAuthPath(pathname: string): boolean {
	return pathname === SIGN_IN_PATH || pathname.startsWith(`${SIGN_IN_PATH}/`);
}

/** Whether the guard needs to check the browser session. */
export function needsSessionCheck(mode: AuthMode, pathname: string): boolean {
	return mode === "supertokens" && !isAuthPath(pathname);
}
