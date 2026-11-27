import { useId } from "react";

import { Switch } from "#/components/ui/switch";
import type { StoreMembership } from "#/lib/api";

/**
 * One switch per store with a rewards program: "I am a member". Turning one
 * changes it at once; the page saves it and offers Undo.
 */
export function MemberPricesForm({
	memberships,
	onChange,
}: {
	memberships: StoreMembership[];
	onChange: (membership: StoreMembership) => void;
}) {
	const id = useId();
	return (
		<ul className="flex flex-col gap-3">
			{memberships.map((membership) => (
				<li key={membership.store} className="flex items-center gap-3">
					<Switch
						id={`${id}-${membership.store}`}
						checked={membership.member}
						onCheckedChange={(member) => onChange({ ...membership, member })}
					/>
					<label htmlFor={`${id}-${membership.store}`} className="text-sm">
						{membership.store_name} {membership.program} member
					</label>
				</li>
			))}
		</ul>
	);
}
