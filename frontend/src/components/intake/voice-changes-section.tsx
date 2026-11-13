import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import { useVoiceChanges } from "#/hooks/useVoiceChanges";
import { formatLoggedAt } from "#/lib/access-log-display";
import type { VoiceListChange } from "#/lib/api";
import { describeChange, undoState } from "#/lib/voice-changes";

/**
 * Items Alexa removed or reduced, newest first. A change under 30 minutes
 * old has **Undo**, which puts the item back as the spoken "undo" would.
 */
export function VoiceChangesSection({
	initial,
}: {
	initial: VoiceListChange[];
}) {
	const { changes, undoing, undo } = useVoiceChanges(initial);

	return (
		<Card role="region" aria-label="Changed by voice">
			<CardHeader>
				<CardTitle className="text-base">Changed by voice</CardTitle>
			</CardHeader>
			<CardContent>
				{changes.length === 0 ? (
					<p className="text-sm text-muted-foreground">
						Nothing has been removed by voice yet.
					</p>
				) : (
					<ul className="flex list-none flex-col gap-3 p-0">
						{changes.map((change) => (
							<VoiceChangeRow
								key={change.id}
								change={change}
								disabled={undoing !== null}
								onUndo={() => undo(change)}
							/>
						))}
					</ul>
				)}
			</CardContent>
		</Card>
	);
}

/** One change: the item, what happened, when, and Undo or why not. */
function VoiceChangeRow({
	change,
	disabled,
	onUndo,
}: {
	change: VoiceListChange;
	disabled: boolean;
	onUndo: () => void;
}) {
	const state = undoState(change);
	return (
		<li
			aria-label={change.item_name}
			className="flex items-center justify-between gap-2"
		>
			<div className="flex min-w-0 flex-col">
				<span className="text-sm font-medium">{change.item_name}</span>
				<span className="text-xs text-muted-foreground">
					{describeChange(change)} · {formatLoggedAt(change.created_at)}
				</span>
			</div>
			{state ? (
				<Badge variant="outline">{state}</Badge>
			) : (
				<Button
					size="sm"
					variant="outline"
					disabled={disabled}
					onClick={onUndo}
					aria-label={`Undo the change to ${change.item_name}`}
				>
					Undo
				</Button>
			)}
		</li>
	);
}
