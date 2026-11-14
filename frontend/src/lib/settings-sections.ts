/**
 * Every settings page, in the order they are listed: one source for the
 * Settings side navigation and the `/settings` index.
 */
import type { LinkProps } from "@tanstack/react-router";

/** One settings page. */
export interface SettingsSection {
	to: LinkProps["to"];
	label: string;
	/** What is set there, in a few words. */
	description: string;
}

export const SETTINGS_SECTIONS: ReadonlyArray<SettingsSection> = [
	{
		to: "/settings/item-rules",
		label: "Item rules",
		description: "Names and amounts fixed when an item is added.",
	},
	{
		to: "/settings/dislikes",
		label: "Dislikes",
		description: "Products each person does not want.",
	},
	{
		to: "/settings/delivery",
		label: "Delivery",
		description: "Delivery fees, how stores are picked, the usual day.",
	},
	{
		to: "/settings/display",
		label: "Display",
		description: "What the product lists show.",
	},
	{
		to: "/settings/logs",
		label: "Logs",
		description: "Who opened the app, and when.",
	},
	{
		to: "/settings/help",
		label: "Help",
		description: "Ways to add items, and changes made by voice.",
	},
];
