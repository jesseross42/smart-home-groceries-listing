import "@tanstack/react-start/server-only";
import SuperTokens from "supertokens-node";
import Session from "supertokens-node/recipe/session";
import ThirdParty from "supertokens-node/recipe/thirdparty";

let initialized = false;

/** Reads required server settings without including their values in errors. */
export function requiredAuthSetting(key: string): string {
	const value = process.env[key]?.trim();
	if (!value) throw new Error(`Missing ${key}`);
	return value;
}

/** Initializes the Core connection and the household OIDC provider. */
export function initServerAuth(): void {
	if (initialized) return;
	const origin = new URL(requiredAuthSetting("AUTH_PUBLIC_ORIGIN"));
	const discovery = new URL(requiredAuthSetting("AUTH_OIDC_DISCOVERY_URL"));
	const loopback =
		origin.hostname === "localhost" || origin.hostname === "127.0.0.1";
	if (
		origin.origin !== origin.href.slice(0, -1) ||
		(origin.protocol !== "https:" &&
			!(loopback && origin.protocol === "http:")) ||
		discovery.protocol !== "https:" ||
		!discovery.pathname.endsWith("/.well-known/openid-configuration") ||
		discovery.username ||
		discovery.password ||
		discovery.search ||
		discovery.hash
	) {
		throw new Error("Invalid authentication URLs");
	}
	const issuer = discovery.href.replace(
		/\.well-known\/openid-configuration$/,
		"",
	);
	SuperTokens.init({
		framework: "custom",
		appInfo: {
			appName: "Grocery List",
			apiDomain: origin.origin,
			websiteDomain: origin.origin,
			apiBasePath: "/api/auth",
			websiteBasePath: "/sign-in",
		},
		supertokens: {
			connectionURI: requiredAuthSetting("SUPERTOKENS_CONNECTION_URI"),
			apiKey: requiredAuthSetting("SUPERTOKENS_API_KEY"),
		},
		recipeList: [
			Session.init({
				getTokenTransferMethod: () => "header",
				override: {
					functions: (original) => ({
						...original,
						async createNewSession(input) {
							const user = await SuperTokens.getUser(
								input.recipeUserId.getAsString(),
								input.userContext,
							);
							return original.createNewSession({
								...input,
								accessTokenPayload: {
									...input.accessTokenPayload,
									email: user?.emails[0],
								},
							});
						},
					}),
				},
			}),
			ThirdParty.init({
				override: {
					apis: (original) => ({
						...original,
						async signInUpPOST(input) {
							if (
								!("redirectURIInfo" in input) ||
								input.redirectURIInfo?.redirectURIOnProviderDashboard !==
									`${origin.origin}/sign-in/callback`
							) {
								return {
									status: "GENERAL_ERROR",
									message: "Invalid sign-in callback",
								};
							}
							if (!original.signInUpPOST)
								throw new Error("Sign-in API is disabled");
							return original.signInUpPOST(input);
						},
						async authorisationUrlGET(input) {
							if (
								input.redirectURIOnProviderDashboard !==
								`${origin.origin}/sign-in/callback`
							) {
								return {
									status: "GENERAL_ERROR",
									message: "Invalid sign-in callback",
								};
							}
							if (!original.authorisationUrlGET)
								throw new Error("Sign-in API is disabled");
							return original.authorisationUrlGET(input);
						},
					}),
				},
				signInAndUpFeature: {
					providers: [
						{
							override: (original) => {
								if (original.type !== "oauth2")
									throw new Error("Expected OIDC provider");
								return {
									...original,
									async getUserInfo(
										input: Parameters<typeof original.getUserInfo>[0],
									) {
										const info = await original.getUserInfo(input);
										const subject =
											info.rawUserInfoFromProvider.fromIdTokenPayload?.sub;
										if (typeof subject !== "string" || !subject) {
											throw new Error("Missing verified OIDC subject");
										}
										return { ...info, thirdPartyUserId: subject };
									},
								};
							},
							config: {
								thirdPartyId: "authentik",
								name: "Household account",
								oidcDiscoveryEndpoint: discovery.href,
								async validateIdTokenPayload({ idTokenPayload }) {
									if (idTokenPayload.iss !== issuer)
										throw new Error("Invalid OIDC issuer");
								},
								requireEmail: true,
								clients: [
									{
										clientId: requiredAuthSetting("AUTH_OIDC_CLIENT_ID"),
										clientSecret: requiredAuthSetting(
											"AUTH_OIDC_CLIENT_SECRET",
										),
										scope: ["openid", "profile", "email"],
										forcePKCE: true,
									},
								],
							},
						},
					],
				},
			}),
		],
	});
	initialized = true;
}
