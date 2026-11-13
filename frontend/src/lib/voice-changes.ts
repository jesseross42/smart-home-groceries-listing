/**
 * How a voice change reads on Settings › Help. Pure, so it is unit-tested.
 */
import { ApiError, type VoiceListChange } from "#/lib/api";

/** "Removed (was ×2)" or "×3 → ×1". */
export function describeChange(
	change: Pick<VoiceListChange, "kind" | "quantity_before" | "quantity_after">,
): string {
	return change.kind === "removed"
		? `Removed (was ×${change.quantity_before})`
		: `×${change.quantity_before} → ×${change.quantity_after}`;
}

/** Why a change has no Undo button, or null when it has one. */
export function undoState(
	change: Pick<VoiceListChange, "undone_at" | "can_undo">,
): "Undone" | "Too old to undo" | null {
	if (change.undone_at) return "Undone";
	return change.can_undo ? null : "Too old to undo";
}

/**
 * What to tell the household when an Undo failed: the backend's own reason
 * for a refusal (409: too old, already undone, item committed), otherwise a
 * plain "try again".
 */
export function undoFailure(error: unknown): string {
	if (error instanceof ApiError && error.status === 409) {
		const detail = (error.body as { detail?: unknown } | null)?.detail;
		if (typeof detail === "string") return `Could not undo: ${detail}.`;
	}
	return "Could not undo that — try again.";
}
