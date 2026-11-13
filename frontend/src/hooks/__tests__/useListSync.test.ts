import { renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useListSync } from "#/hooks/useListSync";
import type { ServerEvent } from "#/lib/ws";

const invalidate = vi.fn();
const connect = vi.fn();
let listener: ((event: ServerEvent) => void) | null = null;
const unsubscribe = vi.fn();

vi.mock("@tanstack/react-router", () => ({
	useRouter: () => ({ invalidate }),
}));
vi.mock("#/lib/ws", () => ({
	connect: () => connect(),
	subscribe: (l: (event: ServerEvent) => void) => {
		listener = l;
		return unsubscribe;
	},
}));

describe("useListSync", () => {
	beforeEach(() => {
		vi.useFakeTimers();
		for (const mock of [invalidate, connect, unsubscribe]) mock.mockReset();
		listener = null;
	});
	afterEach(() => vi.useRealTimers());

	it("reads the page's data again when the list changed elsewhere", () => {
		renderHook(() => useListSync());

		listener?.({ type: "grocery_list_changed" });
		vi.advanceTimersByTime(200);

		expect(connect).toHaveBeenCalled();
		expect(invalidate).toHaveBeenCalledTimes(1);
	});

	it("reads once for a burst of changes", () => {
		renderHook(() => useListSync());

		listener?.({ type: "grocery_list_changed" });
		vi.advanceTimersByTime(100);
		listener?.({ type: "grocery_list_changed" });
		vi.advanceTimersByTime(200);

		expect(invalidate).toHaveBeenCalledTimes(1);
	});

	it("ignores count events", () => {
		renderHook(() => useListSync());

		listener?.({ type: "voice_request_added", count: 2 });
		vi.advanceTimersByTime(500);

		expect(invalidate).not.toHaveBeenCalled();
	});

	it("stops listening, and drops a pending read, when the page closes", () => {
		const { unmount } = renderHook(() => useListSync());
		listener?.({ type: "grocery_list_changed" });

		unmount();
		vi.advanceTimersByTime(500);

		expect(unsubscribe).toHaveBeenCalled();
		expect(invalidate).not.toHaveBeenCalled();
	});
});
