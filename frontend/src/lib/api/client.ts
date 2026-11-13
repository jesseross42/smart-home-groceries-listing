/**
 * The one function that talks to the backend: adds the session token, sends
 * JSON, and turns a non-2xx answer into an {@link ApiError}.
 *
 * Errors arrive as `{ "detail": ... }` — a string for most failures, an
 * object for the duplicate-item 409 — and are kept whole on `ApiError.body`.
 */
import { getAuthToken, refreshAuth } from "#/lib/auth";
import { AUTH_MODE } from "#/lib/auth-mode";
import { API_BASE_URL } from "#/lib/config";
import { SIGN_IN_PATH } from "#/lib/sign-in-guard";

/** A non-2xx answer from the backend, with its parsed body. */
export class ApiError extends Error {
	constructor(
		public status: number,
		public body: unknown,
	) {
		super(typeof body === "string" ? body : JSON.stringify(body));
	}
}

/** Sends a request to the backend and parses the JSON answer. */
export async function request<T>(path: string, init?: RequestInit): Promise<T> {
	let res = await send(path, init);
	if (res.status === 401 && (await refreshAuth())) res = await send(path, init);
	if (
		res.status === 401 &&
		AUTH_MODE === "supertokens" &&
		typeof window !== "undefined"
	) {
		window.location.replace(SIGN_IN_PATH);
	}
	if (!res.ok) {
		const raw = await res.text();
		let body: unknown = raw;
		try {
			body = JSON.parse(raw);
		} catch {
			/* Plain-text error. */
		}
		throw new ApiError(res.status, body);
	}
	if (res.status === 204) return undefined as T;
	return res.json() as Promise<T>;
}

async function send(path: string, init?: RequestInit): Promise<Response> {
	const token = await getAuthToken();
	const headers = new Headers(init?.headers);
	headers.set("Content-Type", "application/json");
	if (token) headers.set("Authorization", `Bearer ${token}`);

	return fetch(`${API_BASE_URL}${path}`, { ...init, headers });
}
