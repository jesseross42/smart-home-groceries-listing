import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";
import {
	loadEnvironment,
	selectedEnvironmentFile,
} from "../scripts/environment.mjs";
import { accepts, component } from "../scripts/service-components.mjs";

/**
 * Resolve frontend configuration before Vite or the auth server reads it. The keys are
 * the production `frontend` component's (`scripts/service-components.mjs`);
 * backend and tunnel credentials are outside this process's boundary.
 */
function loadFrontendEnvironment(): void {
	const frontend = component("frontend");
	Object.assign(
		process.env,
		loadEnvironment({
			...selectedEnvironmentFile(),
			accepts: (key) => accepts(frontend, key),
		}),
	);
}

const config = defineConfig(() => {
	loadFrontendEnvironment();
	return {
		// The component loader is the only dotenv source.
		envDir: false as const,
		resolve: { tsconfigPaths: true },
		plugins: [devtools(), tailwindcss(), tanstackStart(), viteReact(), nitro()],
	};
});

export default config;
