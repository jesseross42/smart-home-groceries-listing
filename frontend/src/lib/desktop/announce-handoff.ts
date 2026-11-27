/**
 * Each trolley fill is announced once, however the news arrives: the Send
 * sheet seeing it filled, the server's `trolley_filled` push, or both, and
 * a push that is repeated.
 */
import { type Notice, notifyDesktop } from "#/lib/desktop/notify";

/** Handoffs already announced in this page. */
const announced = new Set<string>();

/** Shows `notice` for `handoffId` unless that handoff was announced already. */
export function announceHandoff(handoffId: string, notice: Notice): void {
	if (announced.has(handoffId)) return;
	announced.add(handoffId);
	notifyDesktop(notice);
}
