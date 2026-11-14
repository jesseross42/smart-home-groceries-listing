import { useState } from "react";
import { toast } from "sonner";

import { Button } from "#/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { api, type OpenAiKeyStatus } from "#/lib/api";
import { toastWithUndo } from "#/lib/undo-toast";

/** Which key the item checker uses now, in words. */
function keyInUse(status: OpenAiKeyStatus): string {
	if (status.saved) return "A key is saved.";
	if (status.from_environment) return "Using OPENAI_API_KEY from .env.";
	return "No key yet, so every item is held for review.";
}

/**
 * The OpenAI key the item checker uses. The key is saved encrypted on the
 * server and never shown again: it can only be replaced, removed, or the
 * last change undone.
 */
export function OpenAiKeySection({ initial }: { initial: OpenAiKeyStatus }) {
	const [status, setStatus] = useState(initial);
	const [apiKey, setApiKey] = useState("");
	const [busy, setBusy] = useState(false);

	async function change(
		run: () => Promise<OpenAiKeyStatus>,
		done: string,
		failed: string,
	) {
		setBusy(true);
		try {
			setStatus(await run());
			setApiKey("");
			toastWithUndo(done, {
				undo: async () => setStatus(await api.openAiKey.undo()),
				failure: "Could not undo. Save the key again.",
			});
		} catch {
			toast.error(failed);
		} finally {
			setBusy(false);
		}
	}

	function save(event: React.FormEvent) {
		event.preventDefault();
		void change(
			() => api.openAiKey.save(apiKey),
			"OpenAI key saved",
			"Could not save the key — try again.",
		);
	}

	return (
		<Card>
			<CardHeader>
				<CardTitle className="text-base">OpenAI key</CardTitle>
			</CardHeader>
			<CardContent className="flex flex-col gap-3">
				<output className="text-sm">{keyInUse(status)}</output>
				<form
					onSubmit={save}
					aria-label="OpenAI key"
					className="flex flex-col gap-2 sm:flex-row sm:items-end"
				>
					<div className="flex flex-1 flex-col gap-1">
						<Label htmlFor="openai-key">New key</Label>
						<Input
							id="openai-key"
							type="password"
							autoComplete="off"
							value={apiKey}
							onChange={(event) => setApiKey(event.target.value)}
						/>
					</div>
					<Button type="submit" disabled={busy || apiKey.trim() === ""}>
						Save
					</Button>
					{status.saved && (
						<Button
							type="button"
							variant="outline"
							disabled={busy}
							onClick={() =>
								change(
									api.openAiKey.remove,
									"OpenAI key removed",
									"Could not remove the key — try again.",
								)
							}
						>
							Remove
						</Button>
					)}
				</form>
			</CardContent>
		</Card>
	);
}
