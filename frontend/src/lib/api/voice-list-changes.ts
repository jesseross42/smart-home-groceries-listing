/**
 * Items removed or reduced by voice: `/api/voice-list-changes`. Listed on
 * Settings › Help, where one can be undone (`docs/features/FEATURE_VOICE.md`).
 */
import { request } from "#/lib/api/client";

/** One spoken remove or reduce. */
export interface VoiceListChange {
	id: string;
	kind: "removed" | "reduced";
	item_name: string;
	quantity_before: number;
	/** `0` when the item was removed. */
	quantity_after: number;
	created_at: string;
	/** When Undo reversed it; null while it stands. */
	undone_at: string | null;
	/** Still standing and under 30 minutes old. */
	can_undo: boolean;
}

export const voiceListChangesApi = {
	/** The newest changes, newest first. */
	list: () => request<VoiceListChange[]>("/api/voice-list-changes"),
	/** Reverses one change; 409 when it no longer can be. */
	undo: (id: string) =>
		request<VoiceListChange>(`/api/voice-list-changes/${id}/undo`, {
			method: "POST",
		}),
};
