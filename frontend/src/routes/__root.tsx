import { TanStackDevtools } from "@tanstack/react-devtools";
import {
	createRootRoute,
	HeadContent,
	Outlet,
	redirect,
	Scripts,
	useLocation,
} from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import type { ReactNode } from "react";
import { Toaster } from "sonner";

import { SignInOffNotice } from "#/components/auth/sign-in-off-notice";
import { Nav } from "#/components/nav";
import { useDesktopNotifications } from "#/hooks/useDesktopNotifications";
import { useHydrated } from "#/hooks/useHydrated";
import { useTriageToasts } from "#/hooks/useTriageToasts";
import { useVoiceRequestToasts } from "#/hooks/useVoiceRequestToasts";
import { getAuthToken } from "#/lib/auth";
import { AUTH_MODE } from "#/lib/auth-mode";
import { publicConfig, publicConfigScript } from "#/lib/public-config";
import {
	isAuthPath,
	needsSessionCheck,
	SIGN_IN_PATH,
} from "#/lib/sign-in-guard";

import appCss from "#/styles.css?url";

export const Route = createRootRoute({
	head: () => ({
		meta: [
			{ charSet: "utf-8" },
			{ name: "viewport", content: "width=device-width, initial-scale=1" },
			{ title: "Grocery List" },
		],
		links: [
			{ rel: "stylesheet", href: appCss },
			// Without an explicit icon every page load asks for /favicon.ico and
			// logs a 404 in the console.
			{ rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
		],
	}),
	ssr: AUTH_MODE === "off",
	shellComponent: RootDocument,
	beforeLoad: requireSignIn,
	component: RootComponent,
});

/**
 * Sends a signed-out visitor to `/sign-in` before any page loader runs, so
 * no page asks the backend for data it would refuse. Does nothing with
 * sign-in off.
 */
async function requireSignIn({ location }: { location: { pathname: string } }) {
	if (!needsSessionCheck(AUTH_MODE, location.pathname)) return;
	if (!(await getAuthToken())) {
		throw redirect({ href: SIGN_IN_PATH });
	}
}

function RootComponent() {
	const { pathname } = useLocation();
	if (isAuthPath(pathname))
		return (
			<main>
				<Outlet />
			</main>
		);
	return <Household />;
}

function Household() {
	useVoiceRequestToasts();
	useTriageToasts();
	useDesktopNotifications();
	return (
		<>
			<SignInOffNotice />
			<Nav />
			<main className="mx-auto w-full max-w-5xl p-4">
				<Outlet />
			</main>
			<Toaster richColors position="top-right" />
		</>
	);
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
	const hydrated = useHydrated();

	return (
		<html lang="en">
			<head>
				<script>{publicConfigScript(publicConfig())}</script>
				<HeadContent />
			</head>
			<body>
				<div id="app" data-hydrated={hydrated}>
					{children}
				</div>
				<TanStackDevtools
					config={{ position: "bottom-right" }}
					plugins={[
						{
							name: "TanStack Router",
							render: <TanStackRouterDevtoolsPanel />,
						},
					]}
				/>
				<Scripts />
			</body>
		</html>
	);
}
