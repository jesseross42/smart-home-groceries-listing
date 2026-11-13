import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VoiceChangesSection } from "#/components/intake/voice-changes-section";
import type { VoiceListChange } from "#/lib/api";

const list = vi.fn();
const undo = vi.fn();
const toastSuccess = vi.fn();
const toastError = vi.fn();

vi.mock("#/lib/api", async (original) => ({
	...(await original<typeof import("#/lib/api")>()),
	api: {
		voiceChanges: {
			list: () => list(),
			undo: (id: string) => undo(id),
		},
	},
}));
vi.mock("sonner", () => ({
	toast: {
		success: (m: string) => toastSuccess(m),
		error: (m: string) => toastError(m),
	},
}));

function change(overrides: Partial<VoiceListChange> = {}): VoiceListChange {
	return {
		id: "change-1",
		kind: "removed",
		item_name: "milk",
		quantity_before: 2,
		quantity_after: 0,
		created_at: "2026-10-02T04:40:16Z",
		undone_at: null,
		can_undo: true,
		...overrides,
	};
}

describe("VoiceChangesSection", () => {
	beforeEach(() => {
		for (const mock of [list, undo, toastSuccess, toastError]) mock.mockReset();
	});

	it("says when nothing was changed by voice", () => {
		render(<VoiceChangesSection initial={[]} />);

		expect(
			screen.getByText("Nothing has been removed by voice yet."),
		).toBeInTheDocument();
	});

	it("shows each change with what happened", () => {
		render(
			<VoiceChangesSection
				initial={[
					change(),
					change({
						id: "change-2",
						kind: "reduced",
						item_name: "eggs",
						quantity_before: 3,
						quantity_after: 1,
						can_undo: false,
					}),
				]}
			/>,
		);

		expect(screen.getByRole("listitem", { name: "milk" })).toHaveTextContent(
			"Removed (was ×2)",
		);
		const eggs = screen.getByRole("listitem", { name: "eggs" });
		expect(eggs).toHaveTextContent("×3 → ×1");
		expect(eggs).toHaveTextContent("Too old to undo");
		expect(within(eggs).queryByRole("button")).not.toBeInTheDocument();
	});

	it("undoes a change and shows what the backend has now", async () => {
		undo.mockResolvedValue(change({ undone_at: "2026-10-02T04:45:00Z" }));
		list.mockResolvedValue([
			change({ undone_at: "2026-10-02T04:45:00Z", can_undo: false }),
		]);
		render(<VoiceChangesSection initial={[change()]} />);

		await userEvent.click(
			screen.getByRole("button", { name: "Undo the change to milk" }),
		);

		expect(undo).toHaveBeenCalledWith("change-1");
		expect(toastSuccess).toHaveBeenCalledWith("milk is back as it was");
		expect(
			await screen.findByRole("listitem", { name: "milk" }),
		).toHaveTextContent("Undone");
	});

	it("says why an Undo was refused", async () => {
		const { ApiError } = await import("#/lib/api");
		undo.mockRejectedValue(
			new ApiError(409, { detail: "milk has been committed since" }),
		);
		list.mockResolvedValue([change()]);
		render(<VoiceChangesSection initial={[change()]} />);

		await userEvent.click(
			screen.getByRole("button", { name: "Undo the change to milk" }),
		);

		expect(toastError).toHaveBeenCalledWith(
			"Could not undo: milk has been committed since.",
		);
	});
});
