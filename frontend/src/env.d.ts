/**
 * The `VITE_*` values this app reads, from the root `.env`.
 */
interface ImportMetaEnv {
	readonly VITE_API_BASE_URL?: string;
	readonly VITE_WS_URL?: string;
	/** `off` forces sign-in off (e2e tests). See `lib/auth-mode.ts`. */
	readonly VITE_AUTH_MODE?: string;
}
