import { type FormEvent, useId, useState } from "react";

import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import type { AccessLogFilters } from "#/hooks/useAccessLogPages";
import { UNAUTHENTICATED } from "#/lib/access-log-display";

/**
 * The Logs page's filters: one user id, and the start of the path. **Filter**
 * applies both; **Clear** empties both and shows everything again, and
 * stays available while the rows shown are filtered.
 */
export function AccessLogFilterForm({
	disabled = false,
	applied,
	onFilter,
}: {
	disabled?: boolean;
	/** The filters the rows shown were loaded with. */
	applied: AccessLogFilters;
	onFilter: (filters: AccessLogFilters) => void;
}) {
	const userId = useId();
	const pathId = useId();
	const [user, setUser] = useState("");
	const [path, setPath] = useState("");

	function submit(event: FormEvent) {
		event.preventDefault();
		onFilter({ user: user.trim(), path: path.trim() });
	}

	function clear() {
		setUser("");
		setPath("");
		onFilter({ user: "", path: "" });
	}

	return (
		<form
			aria-label="Filter the access log"
			onSubmit={submit}
			className="flex flex-col gap-2 sm:flex-row sm:items-end"
		>
			<div className="flex flex-1 flex-col gap-1">
				<Label htmlFor={userId}>User</Label>
				<Input
					id={userId}
					value={user}
					onChange={(e) => setUser(e.target.value)}
					placeholder={`User id, or ${UNAUTHENTICATED}`}
				/>
			</div>
			<div className="flex flex-1 flex-col gap-1">
				<Label htmlFor={pathId}>Path starts with</Label>
				<Input
					id={pathId}
					value={path}
					onChange={(e) => setPath(e.target.value)}
					placeholder="/api/grocery-items"
				/>
			</div>
			<div className="flex gap-2">
				<Button type="submit" disabled={disabled}>
					Filter
				</Button>
				<Button
					type="button"
					variant="outline"
					disabled={
						disabled || (!user && !path && !applied.user && !applied.path)
					}
					onClick={clear}
				>
					Clear
				</Button>
			</div>
		</form>
	);
}
