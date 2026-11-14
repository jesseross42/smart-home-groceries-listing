import { render, screen, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { SettingsIndex } from "#/components/settings/settings-index";
import { SETTINGS_SECTIONS } from "#/lib/settings-sections";

vi.mock("@tanstack/react-router", () => ({
	Link: ({ to, children }: { to: string; children: ReactNode }) => (
		<a href={to}>{children}</a>
	),
}));

describe("SettingsIndex", () => {
	it("links to every settings page and says what each is for", () => {
		render(<SettingsIndex />);

		const pages = within(screen.getByRole("list", { name: "Settings pages" }));
		expect(pages.getAllByRole("link")).toHaveLength(SETTINGS_SECTIONS.length);
		const delivery = pages.getByRole("link", { name: /Delivery/ });
		expect(delivery).toHaveAttribute("href", "/settings/delivery");
		expect(delivery).toHaveTextContent("Delivery fees");
	});
});
