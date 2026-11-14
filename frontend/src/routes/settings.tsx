import { createFileRoute, Outlet } from "@tanstack/react-router";

import { SectionLayout } from "#/components/section-layout";
import { SETTINGS_SECTIONS } from "#/lib/settings-sections";

export const Route = createFileRoute("/settings")({
	component: SettingsLayout,
});

/** `/settings` — every settings page, with its navigation on the left. */
function SettingsLayout() {
	return (
		<SectionLayout label="Settings sections" links={SETTINGS_SECTIONS}>
			<Outlet />
		</SectionLayout>
	);
}
