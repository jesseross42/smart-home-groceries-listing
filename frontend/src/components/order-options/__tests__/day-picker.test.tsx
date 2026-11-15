import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { DayPicker, withinLabel } from "#/components/order-options/day-picker";

/** Sunday 15 Nov 2026. */
const today = new Date(2026, 10, 15, 11, 0);

describe("DayPicker", () => {
	it("offers today up to 7 days ahead, each with the day's name", async () => {
		const onChange = vi.fn();
		render(
			<DayPicker
				withinDays={null}
				usualDay={null}
				storeFees={[]}
				today={today}
				onChange={onChange}
			/>,
		);

		const picker = screen.getByRole("combobox", { name: "Delivery comes" });
		expect(picker).toHaveTextContent("Within 7 days, by Sunday 22 Nov");
		await userEvent.click(picker);
		const options = screen.getAllByRole("option").map((o) => o.textContent);
		expect(options).toHaveLength(8);
		expect(options[0]).toBe("Today (Sunday)");
		expect(options[2]).toBe("Within 2 days, by Tuesday 17 Nov");
		await userEvent.click(
			screen.getByRole("option", { name: "Today (Sunday)" }),
		);

		expect(onChange).toHaveBeenCalledWith(0);
		expect(screen.getByText(/Fees from Settings/)).toBeInTheDocument();
	});

	it("shows the usual day when no range was picked", () => {
		render(
			<DayPicker
				withinDays={null}
				usualDay="tuesday"
				storeFees={[
					{
						store: "woolworths",
						store_name: "Woolworths",
						date: "2026-11-17",
						fee: "2",
					},
				]}
				today={today}
				onChange={vi.fn()}
			/>,
		);

		expect(
			screen.getByRole("combobox", { name: "Delivery comes" }),
		).toHaveTextContent("On the usual day (Tuesday)");
		expect(screen.getByText(/Woolworths \$2\.00 on Tue/)).toBeInTheDocument();
	});

	it("names one day in the singular", () => {
		expect(withinLabel(1, today)).toBe("Within 1 day, by Monday 16 Nov");
	});
});
