import "@tanstack/react-start/server-only";
import { createHash, timingSafeEqual } from "node:crypto";
import {
	CollectingResponse,
	middleware,
	PreParsedRequest,
} from "supertokens-node/framework/custom";
import Session from "supertokens-node/recipe/session";
import { initServerAuth, requiredAuthSetting } from "./auth-config";

const authMiddleware = middleware();
const VERIFY_PATH = "/api/auth/internal/verify";
const noStore = { "Cache-Control": "no-store" };

function digest(value: string): Buffer {
	return createHash("sha256").update(value).digest();
}

/** Checks a bearer token on behalf of Axum, including revocation in Core. */
async function verify(request: Request): Promise<Response> {
	const secret = request.headers.get("x-grocery-auth-secret");
	if (
		!secret ||
		!timingSafeEqual(
			digest(secret),
			digest(requiredAuthSetting("AUTH_VERIFIER_SECRET")),
		)
	) {
		return new Response(null, { status: 404, headers: noStore });
	}
	const token = request.headers
		.get("authorization")
		?.match(/^Bearer (\S+)$/i)?.[1];
	if (!token) return new Response(null, { status: 401, headers: noStore });
	initServerAuth();
	try {
		const session = await Session.getSessionWithoutRequestResponse(
			token,
			undefined,
			{
				antiCsrfCheck: false,
				checkDatabase: true,
				sessionRequired: true,
			},
		);
		const { email } = session.getAccessTokenPayload();
		return Response.json(
			{
				id: session.getUserId(),
				email: typeof email === "string" ? email : null,
			},
			{ headers: noStore },
		);
	} catch (error) {
		if (!(error instanceof Session.Error)) throw error;
		return new Response(null, { status: 401, headers: noStore });
	}
}

/** Adapts the Fetch request to the SDK's ThirdParty and header-session APIs. */
export async function handleAuth(request: Request): Promise<Response> {
	const url = new URL(request.url);
	try {
		if (url.pathname === VERIFY_PATH && request.method === "POST")
			return await verify(request);
		const origin = requiredAuthSetting("AUTH_PUBLIC_ORIGIN").replace(/\/$/, "");
		const requestOrigin = request.headers.get("origin");
		if (requestOrigin && requestOrigin !== origin) {
			return new Response(null, { status: 403, headers: noStore });
		}
		initServerAuth();
		const response = new CollectingResponse();
		const result = await authMiddleware(
			new PreParsedRequest({
				url: request.url,
				method: request.method === "GET" ? "get" : "post",
				headers: request.headers,
				cookies: {},
				query: Object.fromEntries(url.searchParams),
				getJSONBody: () => request.json(),
				getFormBody: async () => Object.fromEntries(await request.formData()),
			}),
			response,
		);
		if (result.error) throw result.error;
		if (!result.handled)
			return new Response(null, { status: 404, headers: noStore });
		response.headers.set("Cache-Control", "no-store");
		return new Response(response.body, {
			status: response.statusCode,
			headers: response.headers,
		});
	} catch {
		return new Response(null, { status: 503, headers: noStore });
	}
}
