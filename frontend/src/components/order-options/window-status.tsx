import type { OptionStore } from "#/lib/api";

function localTime(value: string, includeDate: boolean): string {
	// API timestamps are wall-clock Sydney times. Format the fields as given.
	return new Intl.DateTimeFormat("en-AU", {
		timeZone: "UTC",
		...(includeDate && { weekday: "short", day: "numeric", month: "short" }),
		hour: "numeric",
		minute: "2-digit",
	}).format(new Date(`${value}Z`));
}

/** Formats the saved deadline using the same clock as the reported windows. */
export function deadlineLabel(deadline: string): string {
	return localTime(deadline, true);
}

const problems = {
	windows_unavailable:
		"Refresh this store’s delivery windows to check this option.",
	no_matching_window:
		"No reported delivery window meets the timing constraints.",
	need_by_passed: "The need-by time has passed.",
};

/** Displays the window this store part can use, or why it cannot be scheduled. */
export function WindowStatus({ store }: { store: OptionStore }) {
	if (store.delivery_problem) {
		return (
			<p className="text-xs text-destructive">
				{problems[store.delivery_problem]}
			</p>
		);
	}
	const window = store.delivery_window;
	if (!window) return null;
	const crossesMidnight = window.start.slice(0, 10) !== window.end.slice(0, 10);
	return (
		<p className="text-xs text-muted-foreground">
			Delivery window {localTime(window.start, true)} to{" "}
			{localTime(window.end, crossesMidnight)}
		</p>
	);
}
