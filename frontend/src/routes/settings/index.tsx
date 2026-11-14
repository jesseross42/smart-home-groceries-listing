import { createFileRoute } from "@tanstack/react-router";

import { SettingsIndex } from "#/components/settings/settings-index";

/** `/settings` before a section is picked: links to every settings page. */
export const Route = createFileRoute("/settings/")({
	component: SettingsIndex,
});
