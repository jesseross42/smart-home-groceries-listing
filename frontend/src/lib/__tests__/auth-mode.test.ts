import { describe, expect, it } from "vitest";
import { resolveAuthMode } from "#/lib/auth-mode";

describe("authentication mode", () => {
	it("requires sign-in by default", () => {
		expect(resolveAuthMode({})).toBe("supertokens");
	});

	it("supports an explicit local bypass", () => {
		expect(resolveAuthMode({ VITE_AUTH_MODE: " OFF " })).toBe("off");
	});
});
