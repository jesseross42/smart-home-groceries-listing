/**
 * Coles' "Choose delivery slot" drawer, built with the markup captured from
 * coles.com.au on 2026-11-14 (trimmed to what the bookmark reads), and
 * behaving like it: picking a day shows that day's times, picking a time
 * enables "Confirm time slot".
 */

/** One delivery time, as the drawer shows it. */
export interface DrawerSlot {
	value: string;
	/** e.g. "9:30am to 10:30am". */
	label: string;
	/** e.g. "$13". */
	fee: string;
	deliverMore?: boolean;
	disabled?: boolean;
}

/** One day button and its times. */
export interface DrawerDay {
	/** e.g. "Tomorrow", "Mon". */
	day: string;
	/** e.g. "15 Nov". */
	date: string;
	slots: DrawerSlot[];
}

/** One time's markup, as Coles renders it. */
export function slotMarkup(slot: DrawerSlot): string {
	const badge = slot.deliverMore
		? '<span data-testid="badge-wrapper" aria-label="Deliver More"><div class="badge-label">Deliver More</div></span>'
		: "";
	return `<div data-testid="slot-window-item" class="coles-targeting-SlotSelectItemContainer"><label><span data-testid="${slot.value}"><input name=":r1d:" type="radio" value="${slot.value}"${slot.disabled ? " disabled" : ""}></span><span><div><div><span class="coles-targeting-SlotSelectItemLabelHeading css-a90g85">${slot.label}</span>${badge}</div><div><span class="coles-targeting-SlotWindowItemSlotWindowItemCost css-1piqg2">${slot.fee}</span></div></div></span></label></div>`;
}

/** Puts the drawer on the page; `onConfirm` hears the confirmed time. */
export function openColesDrawer(
	days: DrawerDay[],
	onConfirm: (value: string) => void = () => {},
): HTMLElement {
	const drawer = document.createElement("section");
	drawer.dataset.testid = "slot-selector-drawer";
	drawer.innerHTML = `<h2 data-testid="drawer-heading">Choose delivery slot</h2>
<fieldset data-testid="date-select">${days
		.map(
			(day, i) =>
				`<div data-testid="date-select-item-${i}"><input id="d${i}" name="shopping-method-date-selector-:r16:" type="radio"${i === 0 ? " checked" : ""}><label for="d${i}"><span data-testid="date-select-item-${i}-day">${day.day}</span><span data-testid="date-select-item-${i}-date">${day.date}</span></label></div>`,
		)
		.join("")}</fieldset>
<div role="radiogroup" data-testid="slots-radio-list"></div>
<div data-testid="submit-window-wrapper"><button aria-label="Confirm time slot" disabled>Confirm</button></div>`;
	const list = drawer.querySelector(
		'[data-testid="slots-radio-list"]',
	) as HTMLElement;
	const confirm = drawer.querySelector("button") as HTMLButtonElement;
	const show = (day: DrawerDay) => {
		list.innerHTML = `<h3>Morning</h3>${day.slots.map(slotMarkup).join("")}`;
		confirm.disabled = true;
		for (const input of list.querySelectorAll("input")) {
			input.addEventListener("click", () => {
				confirm.disabled = false;
			});
		}
	};
	days.forEach((day, i) => {
		drawer.querySelector(`#d${i}`)?.addEventListener("click", () => show(day));
	});
	confirm.addEventListener("click", () => {
		const picked = list.querySelector<HTMLInputElement>("input:checked");
		if (picked) onConfirm(picked.value);
	});
	if (days[0]) show(days[0]);
	document.body.append(drawer);
	return drawer;
}
