/**
 * The OpenAI key the item checker uses: `/api/triage/openai-key`.
 *
 * The key only travels to the server. Every answer says whether a key is
 * set, never what it is.
 */
import { request } from "#/lib/api/client";

/** What the web app may know about the key. */
export interface OpenAiKeyStatus {
	/** A key is saved in the database (it wins over the server's `.env`). */
	saved: boolean;
	/** `OPENAI_API_KEY` is set in `.env`, used while no key is saved. */
	from_environment: boolean;
	changed_at: string | null;
}

const PATH = "/api/triage/openai-key";

export const openAiKeyApi = {
	status: () => request<OpenAiKeyStatus>(PATH),
	/** Saves `apiKey` encrypted, replacing the one before. */
	save: (apiKey: string) =>
		request<OpenAiKeyStatus>(PATH, {
			method: "PUT",
			body: JSON.stringify({ api_key: apiKey }),
		}),
	remove: () => request<OpenAiKeyStatus>(PATH, { method: "DELETE" }),
	/** Undoes the last save or removal. */
	undo: () => request<OpenAiKeyStatus>(`${PATH}/undo`, { method: "POST" }),
};
