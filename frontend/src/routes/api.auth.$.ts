import { createFileRoute } from "@tanstack/react-router";

/** Same-origin SuperTokens APIs and the authenticated Axum verifier. */
export const Route = createFileRoute("/api/auth/$")({
	server: {
		handlers: {
			GET: async ({ request }) =>
				(await import("#/lib/server/auth-api")).handleAuth(request),
			POST: async ({ request }) =>
				(await import("#/lib/server/auth-api")).handleAuth(request),
		},
	},
});
