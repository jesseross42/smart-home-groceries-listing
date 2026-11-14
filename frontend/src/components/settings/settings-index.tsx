import { Link } from "@tanstack/react-router";

import { SETTINGS_SECTIONS } from "#/lib/settings-sections";

/** `/settings` before a section is picked: every settings page, linked. */
export function SettingsIndex() {
	return (
		<div className="flex flex-col gap-4">
			<h1 className="text-lg font-semibold">Settings</h1>
			<ul aria-label="Settings pages" className="flex flex-col gap-2">
				{SETTINGS_SECTIONS.map((section) => (
					<li key={String(section.to)}>
						<Link
							to={section.to}
							className="flex flex-col gap-0.5 rounded-md border px-3 py-2 hover:bg-accent hover:text-accent-foreground"
						>
							<span className="text-sm font-medium">{section.label}</span>
							<span className="text-xs text-muted-foreground">
								{section.description}
							</span>
						</Link>
					</li>
				))}
			</ul>
		</div>
	);
}
