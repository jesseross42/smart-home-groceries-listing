import { createFileRoute } from "@tanstack/react-router";

import { GoogleTasksSection } from "#/components/intake/google-tasks-section";
import { OpenAiKeySection } from "#/components/intake/openai-key-section";
import { OtherChannels } from "#/components/intake/other-channels";
import { VoiceChangesSection } from "#/components/intake/voice-changes-section";
import { useGoogleSignInReturn } from "#/hooks/useGoogleSignInReturn";
import { api } from "#/lib/api";
import { googleReturn } from "#/lib/google-return";

export const Route = createFileRoute("/settings/help")({
	validateSearch: googleReturn,
	// The voice changes are an extra: if they cannot be read the page still
	// loads, with none listed.
	loader: async () => {
		const [settings, voiceChanges, openAiKey] = await Promise.all([
			api.intake.settings(),
			api.voiceChanges.list().catch(() => []),
			api.openAiKey.status().catch(() => null),
		]);
		return { settings, voiceChanges, openAiKey };
	},
	component: HelpPage,
});

/**
 * Settings › Help — how items get in (spec: "Intake channel configuration").
 * Google Tasks is set up here; Alexa, the webhook and the item checker are
 * set in the server's `.env` and only shown. When the checker is OpenAI, its
 * key is saved here. Items Alexa removed or reduced
 * are listed last, each with Undo while it is under 30 minutes old.
 *
 * Google's sign-in returns here (`GOOGLE_REDIRECT_URI`).
 */
function HelpPage() {
	const { settings, voiceChanges, openAiKey } = Route.useLoaderData();
	useGoogleSignInReturn(Route.useSearch());
	const tasks = settings.google_tasks;

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Help</h1>
				<p className="text-sm text-muted-foreground">
					Ways to add items. Each one waits in Pending until someone accepts it.
				</p>
			</div>
			<GoogleTasksSection
				key={`${tasks.connected_at}-${tasks.task_list?.id}`}
				initial={tasks}
			/>
			<OtherChannels settings={settings} />
			{settings.triage.provider === "openai" && openAiKey && (
				<OpenAiKeySection initial={openAiKey} />
			)}
			<VoiceChangesSection initial={voiceChanges} />
		</div>
	);
}
