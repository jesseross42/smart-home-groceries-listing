import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { AccessLogFilterForm } from "#/components/access-logs/access-log-filter-form";

describe("AccessLogFilterForm", () => {
	it("sends both filters, trimmed, on Filter", async () => {
		const onFilter = vi.fn();
		render(<AccessLogFilterForm onFilter={onFilter} />);

		await userEvent.type(screen.getByLabelText("User"), " unauthenticated ");
		await userEvent.type(
			screen.getByLabelText("Path starts with"),
			"/api/voice",
		);
		await userEvent.click(screen.getByRole("button", { name: "Filter" }));

		expect(onFilter).toHaveBeenCalledWith({
			user: "unauthenticated",
			path: "/api/voice",
		});
	});

	it("empties both filters on Clear", async () => {
		const onFilter = vi.fn();
		render(<AccessLogFilterForm onFilter={onFilter} />);
		await userEvent.type(screen.getByLabelText("User"), "user_abc");

		await userEvent.click(screen.getByRole("button", { name: "Clear" }));

		expect(onFilter).toHaveBeenCalledWith({ user: "", path: "" });
		expect(screen.getByLabelText("User")).toHaveValue("");
	});

	it("has nothing to clear when both are empty", () => {
		render(<AccessLogFilterForm onFilter={vi.fn()} />);

		expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
	});
});
