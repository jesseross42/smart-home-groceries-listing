import { type FormEvent, useState } from "react";

import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";

/**
 * "Search with other words": searches both stores for what is typed
 * instead of the item's name and chips. `words` is the search in use ("" =
 * the item's own); "Search for {item}" goes back to it.
 */
export function OtherWordsSearch({
	itemName,
	words,
	onSearch,
}: {
	itemName: string;
	words: string;
	onSearch: (words: string) => void;
}) {
	const [typed, setTyped] = useState(words);

	function submit(event: FormEvent) {
		event.preventDefault();
		onSearch(typed.trim());
	}

	return (
		<form onSubmit={submit} className="flex flex-col gap-2">
			<Label htmlFor="other-words">Search with other words</Label>
			<div className="flex gap-2">
				<Input
					id="other-words"
					type="search"
					maxLength={100}
					placeholder={itemName}
					value={typed}
					onChange={(event) => setTyped(event.target.value)}
				/>
				<Button type="submit" variant="outline" disabled={!typed.trim()}>
					Search
				</Button>
			</div>
			{words && (
				<div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
					<span>Showing products for “{words}”.</span>
					<Button
						type="button"
						size="sm"
						variant="ghost"
						onClick={() => {
							setTyped("");
							onSearch("");
						}}
					>
						Search for {itemName}
					</Button>
				</div>
			)}
		</form>
	);
}
