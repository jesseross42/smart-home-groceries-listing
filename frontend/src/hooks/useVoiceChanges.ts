import { useState } from "react";
import { toast } from "sonner";

import { api, type VoiceListChange } from "#/lib/api";
import { undoFailure } from "#/lib/voice-changes";

/**
 * The voice changes on Settings › Help, and Undo for one of them.
 *
 * After an Undo, or a refused one, the list is read again so each row shows
 * what the backend has now. A refusal's reason (too old, item committed) is
 * shown as the backend worded it.
 */
export function useVoiceChanges(initial: VoiceListChange[]) {
	const [changes, setChanges] = useState(initial);
	const [undoing, setUndoing] = useState<string | null>(null);

	async function reload() {
		try {
			setChanges(await api.voiceChanges.list());
		} catch {
			// The rows shown are stale at worst; the next visit reads again.
		}
	}

	async function undo(change: VoiceListChange) {
		setUndoing(change.id);
		try {
			await api.voiceChanges.undo(change.id);
			toast.success(`${change.item_name} is back as it was`);
		} catch (error) {
			toast.error(undoFailure(error));
		} finally {
			await reload();
			setUndoing(null);
		}
	}

	return { changes, undoing, undo };
}
