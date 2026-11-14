import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { OpenAiKeySection } from "#/components/intake/openai-key-section";
import { api, type OpenAiKeyStatus } from "#/lib/api";

function status(overrides: Partial<OpenAiKeyStatus> = {}): OpenAiKeyStatus {
	return {
		saved: false,
		from_environment: false,
		changed_at: null,
		...overrides,
	};
}

describe("OpenAiKeySection", () => {
	afterEach(() => vi.restoreAllMocks());

	it("says which key is in use without showing it", () => {
		const { rerender } = render(<OpenAiKeySection initial={status()} />);
		expect(screen.getByRole("status")).toHaveTextContent("No key yet");

		rerender(
			<OpenAiKeySection
				key="environment"
				initial={status({ from_environment: true })}
			/>,
		);
		expect(screen.getByRole("status")).toHaveTextContent("OPENAI_API_KEY");
	});

	it("saves a typed key, clears the field and offers Remove", async () => {
		const save = vi
			.spyOn(api.openAiKey, "save")
			.mockResolvedValue(status({ saved: true }));
		const user = userEvent.setup();
		render(<OpenAiKeySection initial={status()} />);

		expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
		await user.type(screen.getByLabelText("New key"), "sk-secret");
		await user.click(screen.getByRole("button", { name: "Save" }));

		expect(save).toHaveBeenCalledWith("sk-secret");
		expect(await screen.findByText("A key is saved.")).toBeInTheDocument();
		expect(screen.getByLabelText("New key")).toHaveValue("");
		expect(screen.getByRole("button", { name: "Remove" })).toBeEnabled();
	});

	it("removes the saved key", async () => {
		const remove = vi
			.spyOn(api.openAiKey, "remove")
			.mockResolvedValue(status({ from_environment: true }));
		const user = userEvent.setup();
		render(<OpenAiKeySection initial={status({ saved: true })} />);

		await user.click(screen.getByRole("button", { name: "Remove" }));

		expect(remove).toHaveBeenCalled();
		expect(await screen.findByText(/OPENAI_API_KEY/)).toBeInTheDocument();
		expect(screen.queryByRole("button", { name: "Remove" })).toBeNull();
	});
});
