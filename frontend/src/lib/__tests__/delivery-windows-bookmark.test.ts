import { expect, it, vi } from "vitest";
import { deliveryInfo, liveDays } from "#/lib/__tests__/woolworths-fixtures";
import { deliveryWindowsBookmarklet } from "#/lib/store-tab/bookmarklet";
import { WOOLWORTHS_TAB } from "#/lib/store-tab/store-tabs";

it("runs the serialized bookmark and publishes windows without a handoff or reservation", async () => {
	const config = { apiBaseUrl: "https://grocery.test", secret: "test-secret" };
	const bookmark = deliveryWindowsBookmarklet(WOOLWORTHS_TAB, config);
	const program = decodeURIComponent(bookmark.slice("javascript:".length));
	const alert = vi.fn();
	const calls = vi.fn(async (path: string, _init?: RequestInit) => {
		if (path === "/apis/ui/Delivery/DeliveryInfo")
			return Response.json(deliveryInfo());
		if (path.startsWith("/api/v3/ui/fulfilment/windows?")) {
			return Response.json({ Days: liveDays() });
		}
		return new Response(null, { status: 204 });
	});

	new Function("location", "fetch", "alert", program)(
		{ hostname: "www.woolworths.com.au" },
		calls,
		alert,
	);

	await vi.waitFor(() =>
		expect(alert).toHaveBeenCalledWith(expect.stringContaining("Updated")),
	);
	expect(calls).toHaveBeenCalledTimes(3);
	const [path, init] = calls.mock.calls[2];
	expect(path).toBe("https://grocery.test/api/store-tab/delivery-windows");
	expect(init?.method).toBe("POST");
	expect(JSON.parse(String(init?.body))).toEqual({
		secret: "test-secret",
		store: "woolworths",
		windows: expect.arrayContaining([
			{ start: "2026-10-03T04:00:00", end: "2026-10-03T07:00:00", fee: "15" },
		]),
	});
});
