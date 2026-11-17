import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DeliverySettingsForm } from "#/components/delivery/delivery-settings-form";
import type { DeliverySettings } from "#/lib/api";

const unset: DeliverySettings = {
	stores: [
		{
			store: "woolworths",
			store_name: "Woolworths",
			delivery_fee: null,
			free_delivery_over: null,
			minimum_order: null,
			delivery_address: null,
		},
		{
			store: "coles",
			store_name: "Coles",
			delivery_fee: "10",
			free_delivery_over: null,
			minimum_order: null,
			delivery_address: null,
		},
	],
	mode: "minimise_total",
	max_delivery_spend: null,
	default_delivery_day: null,
	allowed_delivery_days: [],
};

function renderForm(onSave = vi.fn().mockResolvedValue(undefined)) {
	render(<DeliverySettingsForm settings={unset} onSave={onSave} />);
	return onSave;
}

describe("DeliverySettingsForm", () => {
	it("shows each store's saved amounts and the chosen mode", () => {
		renderForm();

		const coles = screen.getByRole("group", { name: "Coles" });
		expect(coles).toBeInTheDocument();
		expect(screen.getAllByLabelText("Delivery fee")[1]).toHaveValue("10");
		expect(screen.getByRole("radio", { name: "Cheapest total" })).toBeChecked();
	});

	it("saves typed amounts and a new mode", async () => {
		const onSave = renderForm();

		await userEvent.type(screen.getAllByLabelText("Delivery fee")[0], "9");
		await userEvent.type(
			screen.getAllByLabelText("Free delivery from")[0],
			"$250",
		);
		await userEvent.click(screen.getByRole("radio", { name: "Coles only" }));
		await userEvent.type(
			screen.getByLabelText("Most to spend on delivery"),
			"15",
		);
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith({
			stores: [
				{
					store: "woolworths",
					delivery_fee: "9",
					free_delivery_over: "250",
					minimum_order: null,
					delivery_address: null,
				},
				{
					store: "coles",
					delivery_fee: "10",
					free_delivery_over: null,
					minimum_order: null,
					delivery_address: null,
				},
			],
			mode: "coles_only",
			max_delivery_spend: "15",
			default_delivery_day: null,
			allowed_delivery_days: [],
			need_by: null,
			perishable_hours: null,
		});
	});

	it("saves the deadline and the maximum early arrival for perishables", async () => {
		const onSave = renderForm();
		fireEvent.change(screen.getByLabelText("Need by (Sydney time)"), {
			target: { value: "2026-11-20T18:00" },
		});
		await userEvent.type(
			screen.getByLabelText("Perishables can arrive up to"),
			"4",
		);
		await userEvent.click(screen.getByRole("button", { name: "Save" }));
		expect(onSave).toHaveBeenCalledWith(
			expect.objectContaining({
				need_by: "2026-11-20T18:00:00",
				perishable_hours: 4,
			}),
		);
	});

	it("saves the usual delivery day that was picked", async () => {
		const onSave = renderForm();

		expect(
			screen.getByRole("button", { name: "No usual day" }),
		).toHaveAttribute("aria-pressed", "true");
		await userEvent.click(
			within(
				screen.getByRole("group", { name: "Usual delivery day" }),
			).getByRole("button", { name: "Saturday" }),
		);
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith(
			expect.objectContaining({ default_delivery_day: "saturday" }),
		);
	});

	it("saves the allowed delivery days, and pressing a day again takes it out", async () => {
		const onSave = renderForm();
		const allowed = within(
			screen.getByRole("group", { name: "Allowed delivery days" }),
		);

		await userEvent.click(allowed.getByRole("button", { name: "Tuesday" }));
		await userEvent.click(allowed.getByRole("button", { name: "Friday" }));
		await userEvent.click(allowed.getByRole("button", { name: "Friday" }));
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(onSave).toHaveBeenCalledWith(
			expect.objectContaining({ allowed_delivery_days: ["tuesday"] }),
		);
	});

	it("refuses a usual day that is not an allowed day", async () => {
		const onSave = renderForm();
		const group = (name: string) => within(screen.getByRole("group", { name }));

		await userEvent.click(
			group("Usual delivery day").getByRole("button", { name: "Saturday" }),
		);
		await userEvent.click(
			group("Allowed delivery days").getByRole("button", { name: "Tuesday" }),
		);
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(screen.getByRole("alert")).toHaveTextContent("allowed days");
		expect(onSave).not.toHaveBeenCalled();
	});

	it("explains a wrong amount and sends nothing", async () => {
		const onSave = renderForm();

		await userEvent.type(screen.getAllByLabelText("Minimum order")[0], "lots");
		await userEvent.click(screen.getByRole("button", { name: "Save" }));

		expect(screen.getByRole("alert")).toHaveTextContent("Woolworths");
		expect(onSave).not.toHaveBeenCalled();
	});
});
