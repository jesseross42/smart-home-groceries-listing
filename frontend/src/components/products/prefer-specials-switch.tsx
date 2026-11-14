import { useId } from "react";

import { Switch } from "#/components/ui/switch";
import { useDisplaySettings } from "#/hooks/useDisplaySettings";
import { saveDisplay } from "#/lib/display-settings";

/**
 * "Prefer specials", at the top of the Select item page: lists products on
 * special or with a multibuy first. Saved in this browser (Settings ›
 * Display). It only reorders the list; a chosen product never changes.
 */
export function PreferSpecialsSwitch() {
	const id = useId();
	const display = useDisplaySettings();
	return (
		<div className="flex items-center gap-2">
			<Switch
				id={id}
				checked={display.preferSpecials}
				onCheckedChange={(preferSpecials) =>
					saveDisplay({ ...display, preferSpecials })
				}
			/>
			<label htmlFor={id} className="text-sm">
				Prefer specials
			</label>
		</div>
	);
}
