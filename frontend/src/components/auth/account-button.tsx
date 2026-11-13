import { useState } from "react";
import Session from "supertokens-web-js/recipe/session";
import { Button } from "#/components/ui/button";
import { AUTH_MODE } from "#/lib/auth-mode";
import { SIGN_IN_PATH } from "#/lib/sign-in-guard";

/** Signs out the household member and clears the browser session. */
export function AccountButton() {
	const [error, setError] = useState(false);
	if (AUTH_MODE === "off") return null;

	async function signOut() {
		try {
			await Session.signOut();
			window.location.replace(SIGN_IN_PATH);
		} catch {
			setError(true);
		}
	}

	return (
		<div className="ml-auto flex shrink-0 items-center gap-2 pl-2">
			{error && <span role="alert">Sign-out failed</span>}
			<Button variant="outline" onClick={() => void signOut()}>
				Sign out
			</Button>
		</div>
	);
}
