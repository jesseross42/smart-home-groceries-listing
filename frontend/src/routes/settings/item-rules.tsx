import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { ItemRuleCard } from "#/components/item-rules/item-rule-card";
import { ItemRuleForm } from "#/components/item-rules/item-rule-form";
import { ItemRuleList } from "#/components/item-rules/item-rule-list";
import { Card } from "#/components/ui/card";
import { api, type ItemRuleDraft } from "#/lib/api";
import { toastWithUndo } from "#/lib/undo-toast";

export const Route = createFileRoute("/settings/item-rules")({
	loader: () => api.itemRules.list(),
	component: ItemRulesPage,
});

/**
 * Item Rules — persistent filters attached to grocery item names (spec:
 * "Item Rules > Rule configuration").
 *
 * A rule for "toilet paper" with the filter "3 ply" means an accepted
 * "Hey Google, add toilet paper" lands on the list already chipped "3 ply".
 * The page lists every rule with inline edit and delete, and an add form.
 *
 * Mutations go through `lib/api` and then `router.invalidate()`, so the loader
 * stays the single source of truth.
 */
function ItemRulesPage() {
	const rules = Route.useLoaderData();
	const router = useRouter();

	async function createRule(draft: ItemRuleDraft) {
		try {
			await api.itemRules.create(draft);
			toast.success(`Rule added for ${draft.triggers.join(", ")}`);
			await router.invalidate();
		} catch (error) {
			toast.error("Could not add that rule — try again.");
			throw error;
		}
	}

	async function saveRule(id: string, draft: ItemRuleDraft) {
		try {
			await api.itemRules.update(id, draft);
			toast.success("Rule saved");
			await router.invalidate();
		} catch (error) {
			toast.error("Could not save that rule — try again.");
			throw error;
		}
	}

	async function removeRule(id: string) {
		try {
			await api.itemRules.remove(id);
			toast.success("Rule deleted");
			await router.invalidate();
		} catch {
			toast.error("Could not delete that rule — try again.");
		}
	}

	/** Adds a rule's filters to the list's matching items, with Undo. */
	async function applyRule(id: string) {
		try {
			const { changed } = await api.itemRules.applyToList(id);
			if (changed.length === 0) {
				toast.info("No items on the list needed these filters");
				return;
			}
			const count = changed.length === 1 ? "1 item" : `${changed.length} items`;
			toastWithUndo(`Filters added to ${count}`, {
				description: changed.map((item) => item.name).join(", "),
				undo: () =>
					Promise.all(
						changed.map((item) =>
							api.grocery.update(item.id, {
								filter_terms: item.previous_filter_terms,
							}),
						),
					),
				failure: "Could not undo — check the list.",
			});
		} catch {
			toast.error("Could not apply that rule — try again.");
		}
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Item Rules</h1>
				<p className="text-sm text-muted-foreground">
					Give items filters automatically. A rule for "toilet paper" with the
					filter "3 ply" narrows its product search to 3-ply whenever it is
					added. "Apply to list" adds a rule's filters to items already on the
					list.
				</p>
			</div>

			<Card>
				<h2 className="px-4 pt-4 text-sm font-semibold">New rule</h2>
				<ItemRuleForm
					idPrefix="new-rule"
					submitLabel="Add rule"
					onSubmit={createRule}
				/>
			</Card>

			<ItemRuleList>
				{rules.length === 0 ? (
					<ItemRuleList.Empty>
						No rules yet. Add one above to put filters on an item every time it
						is added.
					</ItemRuleList.Empty>
				) : (
					rules.map((rule) => (
						<ItemRuleList.Item key={rule.id}>
							<ItemRuleCard
								rule={rule}
								onSave={saveRule}
								onRemove={removeRule}
								onApply={applyRule}
							/>
						</ItemRuleList.Item>
					))
				)}
			</ItemRuleList>
		</div>
	);
}
