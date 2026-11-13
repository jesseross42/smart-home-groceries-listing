/**
 * WebSocket client for real-time push events: `voice_request_added` (the
 * Pending Requests count), `triage_held` (the held-for-review count) and
 * `grocery_list_changed` (read the list again). Auto-reconnects with backoff; safe to call
 * `connect` once at app mount — it is a no-op if a socket is already open.
 */
import { getAuthToken } from "#/lib/auth";
import { AUTH_MODE } from "#/lib/auth-mode";
import { WS_URL } from "#/lib/config";
import { socketProtocols } from "#/lib/ws-url";

export interface VoiceRequestAddedEvent {
	type: "voice_request_added";
	count: number;
}

/** The held-for-review count changed. */
export interface TriageHeldEvent {
	type: "triage_held";
	count: number;
}

/** Live changes to a queue count. */
type CountEvent = VoiceRequestAddedEvent | TriageHeldEvent;

/** Initial counts on each connection, including reconnects. Never a new-item notice. */
export interface CountsSnapshotEvent {
	type: "counts_snapshot";
	counts: Record<CountEventType, number>;
}

/**
 * The grocery list, its chosen products or its order may have changed
 * somewhere else (another tab, another person, Alexa). It carries nothing:
 * pages read their data again.
 */
export interface GroceryListChangedEvent {
	type: "grocery_list_changed";
}

/** Every event the backend pushes, including connection baselines. */
export type ServerEvent =
	| CountEvent
	| CountsSnapshotEvent
	| GroceryListChangedEvent;

/** The `type` of an event that carries a count. */
export type CountEventType = CountEvent["type"];

/**
 * The `eventType` count `event` carries: from a connection snapshot or a live
 * event of that type. `null` when the event says nothing about it.
 */
export function countIn(
	event: ServerEvent,
	eventType: CountEventType,
): number | null {
	if (event.type === "counts_snapshot") return event.counts[eventType];
	if (event.type === "grocery_list_changed") return null;
	return event.type === eventType ? event.count : null;
}

type Listener = (event: ServerEvent) => void;

const listeners = new Set<Listener>();
let socket: WebSocket | null = null;
let retryDelayMs = 1000;
let retryTimer: ReturnType<typeof setTimeout> | null = null;
/** True while a session token is being fetched for the next socket. */
let opening = false;

function scheduleReconnect() {
	if (retryTimer) return;
	retryTimer = setTimeout(() => {
		retryTimer = null;
		connect();
	}, retryDelayMs);
	retryDelayMs = Math.min(retryDelayMs * 2, 15000);
}

/**
 * Opens the socket unless one is open, opening, or waiting to retry. With
 * sign-in on, a fresh session token is fetched for every attempt; signed out, it waits and tries again later.
 */
export function connect(): void {
	if (typeof window === "undefined") return; // SSR guard — client only
	if (opening || retryTimer) return;
	if (
		socket &&
		(socket.readyState === WebSocket.OPEN ||
			socket.readyState === WebSocket.CONNECTING)
	) {
		return;
	}

	opening = true;
	getAuthToken()
		.catch(() => null)
		.then((token) => {
			opening = false;
			if (AUTH_MODE === "supertokens" && !token) {
				scheduleReconnect();
				return;
			}
			open(WS_URL, socketProtocols(token));
		});
}

/** Opens one socket and wires its handlers. */
function open(url: string, protocols: string[]): void {
	socket = new WebSocket(url, protocols);

	socket.onopen = () => {
		retryDelayMs = 1000;
	};

	socket.onmessage = (event) => {
		try {
			const parsed = JSON.parse(event.data) as ServerEvent;
			for (const listener of listeners) listener(parsed);
		} catch {
			// Ignore malformed frames rather than crash the socket handler.
		}
	};

	socket.onclose = () => {
		socket = null;
		scheduleReconnect();
	};

	socket.onerror = () => {
		socket?.close();
	};
}

export function subscribe(listener: Listener): () => void {
	listeners.add(listener);
	return () => listeners.delete(listener);
}
