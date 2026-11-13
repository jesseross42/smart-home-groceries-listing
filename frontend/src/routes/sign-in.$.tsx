import { createFileRoute, redirect } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import ThirdParty from "supertokens-web-js/recipe/thirdparty";
import { Button } from "#/components/ui/button";
import { initAuth } from "#/lib/auth";
import { AUTH_MODE } from "#/lib/auth-mode";

export const Route = createFileRoute("/sign-in/$")({
	beforeLoad: () => {
		if (AUTH_MODE === "off") throw redirect({ to: "/" });
	},
	head: () => ({ meta: [{ title: "Sign in · Grocery List" }] }),
	component: SignInPage,
});

/** Starts Authentik sign-in or completes the SDK's authorization-code exchange. */
function SignInPage() {
	const callback = Route.useParams()._splat === "callback";
	const started = useRef(false);
	const [busy, setBusy] = useState(callback);
	const [error, setError] = useState("");

	useEffect(() => {
		if (!callback || started.current) return;
		started.current = true;
		initAuth();
		void ThirdParty.signInAndUp()
			.then((result) => {
				if (result.status !== "OK") throw new Error();
				window.location.replace("/");
			})
			.catch(() => {
				setError(
					"Sign-in could not be completed. Try again or check your household access.",
				);
				setBusy(false);
			});
	}, [callback]);

	async function signIn() {
		setBusy(true);
		setError("");
		try {
			initAuth();
			const url =
				await ThirdParty.getAuthorisationURLWithQueryParamsAndSetState({
					thirdPartyId: "authentik",
					frontendRedirectURI: `${window.location.origin}/sign-in/callback`,
				});
			window.location.assign(url);
		} catch {
			setError("Sign-in could not be started. Try again.");
			setBusy(false);
		}
	}

	return (
		<section className="mx-auto flex max-w-sm flex-col gap-4 py-8">
			<h1 className="text-xl font-semibold">Sign in to Grocery List</h1>
			{error && <p role="alert">{error}</p>}
			<Button disabled={busy} onClick={() => void signIn()}>
				{busy ? "Signing in…" : "Continue with household account"}
			</Button>
		</section>
	);
}
