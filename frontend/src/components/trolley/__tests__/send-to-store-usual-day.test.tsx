import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SendToStore } from "#/components/trolley/send-to-store";
import { api, type StoreId, type TrolleyHandoff } from "#/lib/api";
import { DELIVERY_WEEKDAYS, localDate } from "#/lib/delivery-days";

/** Three days from now: never tomorrow, always one of the days offered. */
function inThreeDays(): Date {
	const now = new Date();
	return new Date(now.getFullYear(), now.getMonth(), now.getDate() + 3);
}

async function sendWithUsualDay(store: StoreId) {
	vi.spyOn(api.trolleyHandoffs, "storeTabSecret").mockResolvedValue({
		secret: "s",
	});
	const create = vi
		.spyOn(api.trolleyHandoffs, "create")
		.mockResolvedValue({
			id: "h1",
			store,
			lines: [],
		} as unknown as TrolleyHandoff);
	vi.spyOn(window, "open").mockReturnValue(null);
	const user = userEvent.setup();
	render(
		<SendToStore
			store={store}
			chosenCount={1}
			usualDay={DELIVERY_WEEKDAYS[inThreeDays().getDay()].day}
		>
			<SendToStore.Trigger />
			<SendToStore.Content />
		</SendToStore>,
	);
	await user.click(screen.getByRole("button", { name: /^Send to/ }));
	return { create, user };
}

describe("SendToStore with a usual delivery day", () => {
	afterEach(() => vi.restoreAllMocks());

	it("starts the Woolworths day chooser on the usual day, and it can be changed", async () => {
		const { create, user } = await sendWithUsualDay("woolworths");

		const tomorrow = screen.getByRole("button", { name: /^Tomorrow/ });
		expect(tomorrow).toHaveAttribute("aria-pressed", "false");
		await user.click(
			screen.getByRole("button", { name: "Send and open Woolworths" }),
		);
		expect(create).toHaveBeenCalledWith("woolworths", {
			date: localDate(inThreeDays()),
			time_of_day: "any",
		});
	});

	it("asks Coles for no day, because its bookmark reserves none", async () => {
		const { create, user } = await sendWithUsualDay("coles");

		await user.click(
			screen.getByRole("button", { name: "Send and open Coles" }),
		);
		expect(create).toHaveBeenCalledWith("coles", {
			date: null,
			time_of_day: "any",
		});
	});
});
