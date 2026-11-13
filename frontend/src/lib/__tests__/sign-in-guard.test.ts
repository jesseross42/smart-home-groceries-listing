import { expect, it } from "vitest";
import { isAuthPath, needsSessionCheck } from "#/lib/sign-in-guard";

it("allows sign-in and its callback without a session", () => {
	expect(needsSessionCheck("supertokens", "/sign-in")).toBe(false);
	expect(needsSessionCheck("supertokens", "/sign-in/callback")).toBe(false);
});

it("requires a session on household pages unless local bypass is enabled", () => {
	expect(isAuthPath("/sign-inside")).toBe(false);
	expect(needsSessionCheck("supertokens", "/pending")).toBe(true);
	expect(needsSessionCheck("off", "/pending")).toBe(false);
});
