import { expect, it } from "vitest";
import { socketProtocols } from "#/lib/ws-url";

it("offers the event protocol separately from the session credential", () => {
	expect(socketProtocols("abc.def.sig")).toEqual([
		"grocery.events",
		"grocery.session.abc.def.sig",
	]);
	expect(socketProtocols(null)).toEqual(["grocery.events"]);
});
