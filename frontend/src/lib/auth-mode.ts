import { publicConfig } from "#/lib/public-config";

/** Authentication is enabled unless local development explicitly disables it. */
export type AuthMode = "supertokens" | "off";

/** Resolves the browser's authentication mode. */
export function resolveAuthMode(env: { VITE_AUTH_MODE?: string }): AuthMode {
	return env.VITE_AUTH_MODE?.trim().toLowerCase() === "off"
		? "off"
		: "supertokens";
}

/** Authentication mode for this instance. */
export const AUTH_MODE = resolveAuthMode(publicConfig());
