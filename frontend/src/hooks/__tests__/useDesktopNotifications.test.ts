import { renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { TrolleyHandoff } from "#/lib/api";
import type { ServerEvent } from "#/lib/ws";

const { listeners, notify, desktopState } = vi.hoisted(() => ({
	listeners: new Set<(event: ServerEvent) => void>(),
	notify: vi.fn(),
	desktopState: { on: true },
}));

vi.mock("#/lib/ws", async (original) => ({
	countIn: (await original<typeof import("#/lib/ws")>()).countIn,
	connect: vi.fn(),
	subscribe: (listener: (event: ServerEvent) => void) => {
		listeners.add(listener);
		return () => listeners.delete(listener);
	},
}));
vi.mock("#/lib/desktop/bridge", () => ({ isDesktop: () => desktopState.on }));
vi.mock("#/lib/desktop/commands", () => ({ desktop: { notify } }));

import {
	HELD_ITEM_NOTICE,
	NEW_ITEM_NOTICE,
	useDesktopNotifications,
} from "#/hooks/useDesktopNotifications";
import { useHandoffNotice } from "#/hooks/useHandoffNotice";
import { resetAnnouncedHandoffs } from "#/lib/desktop/handoff-notice";

function push(event: ServerEvent) {
	for (const listener of listeners) listener(event);
}

describe("desktop notifications", () => {
	beforeEach(() => {
		listeners.clear();
		notify.mockReset().mockResolvedValue(undefined);
		desktopState.on = true;
		resetAnnouncedHandoffs();
	});
	afterEach(() => vi.clearAllMocks());

	it("notifies when a new item arrives or one is held", () => {
		renderHook(() => useDesktopNotifications());
		push({
			type: "counts_snapshot",
			counts: { voice_request_added: 0, triage_held: 0 },
		});
		expect(notify).not.toHaveBeenCalled();
		push({ type: "voice_request_added", count: 1 });
		push({ type: "triage_held", count: 1 });

		expect(notify.mock.calls).toEqual([
			[NEW_ITEM_NOTICE.title, NEW_ITEM_NOTICE.body],
			[HELD_ITEM_NOTICE.title, HELD_ITEM_NOTICE.body],
		]);
	});

	it("silently resets both baselines on reconnect and notices the next rises", () => {
		renderHook(() => useDesktopNotifications());
		push({
			type: "counts_snapshot",
			counts: { voice_request_added: 2, triage_held: 3 },
		});
		push({
			type: "counts_snapshot",
			counts: { voice_request_added: 5, triage_held: 1 },
		});
		expect(notify).not.toHaveBeenCalled();
		push({ type: "voice_request_added", count: 6 });
		push({ type: "triage_held", count: 2 });
		expect(notify.mock.calls).toEqual([
			[NEW_ITEM_NOTICE.title, NEW_ITEM_NOTICE.body],
			[HELD_ITEM_NOTICE.title, HELD_ITEM_NOTICE.body],
		]);
	});

	it("stays quiet in a normal browser", () => {
		desktopState.on = false;
		renderHook(() => useDesktopNotifications());
		push({ type: "voice_request_added", count: 0 });
		push({ type: "voice_request_added", count: 1 });
		expect(notify).not.toHaveBeenCalled();
	});

	it("announces a filled trolley once", () => {
		const waiting = {
			id: "h1",
			status: "claimed_by_store_tab",
			store_name: "Woolworths",
			lines: [],
		} as unknown as TrolleyHandoff;
		const { rerender } = renderHook(
			({ handoff }) => useHandoffNotice(handoff),
			{ initialProps: { handoff: waiting } },
		);
		expect(notify).not.toHaveBeenCalled();

		const filled = { ...waiting, status: "filled" } as TrolleyHandoff;
		rerender({ handoff: filled });
		rerender({ handoff: { ...filled } });

		expect(notify).toHaveBeenCalledTimes(1);
		expect(notify).toHaveBeenCalledWith(
			"Woolworths trolley filled",
			"0 added. Check out when ready.",
		);
	});

	const FILLED: ServerEvent = {
		type: "trolley_filled",
		handoff_id: "h1",
		store_name: "Woolworths",
		added: 3,
		failed: 1,
	};

	it("announces a trolley the server says is filled, with no sheet open", () => {
		renderHook(() => useDesktopNotifications());

		push(FILLED);

		expect(notify.mock.calls).toEqual([
			[
				"Woolworths trolley filled",
				"3 added. 1 not added. Check out when ready.",
			],
		]);
	});

	it("announces once when the same event arrives again", () => {
		renderHook(() => useDesktopNotifications());

		push(FILLED);
		push(FILLED);

		expect(notify).toHaveBeenCalledTimes(1);
	});

	it("announces once when the sheet and the server both report a fill", () => {
		renderHook(() => useDesktopNotifications());
		push(FILLED);

		const filled = {
			id: "h1",
			status: "filled_with_problems",
			store_name: "Woolworths",
			lines: [],
		} as unknown as TrolleyHandoff;
		renderHook(() => useHandoffNotice(filled));

		expect(notify).toHaveBeenCalledTimes(1);
	});

	it("announces a later handoff after a reconnect's snapshot", () => {
		renderHook(() => useDesktopNotifications());
		push(FILLED);
		push({
			type: "counts_snapshot",
			counts: { voice_request_added: 0, triage_held: 0 },
		});

		push({ ...FILLED, handoff_id: "h2" });

		expect(notify).toHaveBeenCalledTimes(2);
	});
});
