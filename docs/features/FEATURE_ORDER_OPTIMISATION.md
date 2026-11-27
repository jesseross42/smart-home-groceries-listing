# Feature: Order Optimisation

Status: **built — the order review, delivery fees, the modes and the
split-store planner (`/order`, `/settings/delivery`).** Fees per day come from the
bookmark (issue #70). Delivery constraints include allowed weekdays, a
need-by date and time, and a freshness interval for marked perishables (issue #74).

## Ways to buy (the planner)
An item may have a chosen product at **each** store
(`FEATURE_PRODUCT_CHOICE.md`); one of them is the one the order buys. The
planner never picks a product: it only moves an item between products already
chosen (by a person, or by auto-pick when the item was added). An item a store
does not sell is never planned at that store.

1. `/order` shows **Ways to buy** above the review. Up to four options, best
   first: **All at Woolworths**, **All at Coles**, **Split between stores**
   (the best mix found) and **Your current stores**. Two that buy the same
   items at the same stores are shown once; the one the order buys now is
   badged **Current**.
2. Each option: total with delivery, each store's items subtotal + delivery
   ("free delivery", "delivery fee not set"), the items it leaves out and
   why, and notes (under a store's minimum, over the delivery cap, fees not
   set).
   A disliked product the option buys is named with who disliked it
   ("milk at Coles: phu disliked this item previously."). The best mix skips
   a disliked product when another store has one nobody dislikes
   (`FEATURE_DISLIKES.md`).
3. **Recommended** marks the first option only when it buys every item,
   meets every store's minimum, stays within the cap and fits the saved delivery constraints.
4. When the recommended option is not the current one, `/order` applies it by
   itself once per plan (`useApplyBestOption`), with a toast and **Undo**.
   An undone or replaced plan is not re-applied until the plan changes.
5. To override, each option that is not current has a short switch button
   (**Switch to Woolworths**, **Switch to Coles**, **Switch to split**,
   **Switch back**) with an (i) next to it; hovering or focusing the (i)
   shows what it does (`switch-button.tsx`, `ui/tooltip.tsx`). It sends the option's `picks` to `PUT /api/order-stores`; the
   toast's **Undo** puts every moved item back. Send to Woolworths/Coles then
   sends only items bought at that store.
6. **Rank by** (`?mode=`) tries another mode for this visit; the saved
   default is on Settings › Delivery.
7. **Delivery comes** is a dropdown: **Today (Sunday)** or **Within N
   days, by <weekday> <date>**, N = 1–7 (`?within=N`). Each store's fee is
   the cheapest it showed from today to that day. Nothing picked: the usual
   day (Settings › Delivery), shown as **On the usual day (<weekday>)**, with
   the fee on the first coming date of that weekday; no usual day = within 7
   days. Fees come from `store_delivery_day_fees`; a store with nothing
   reported in the range keeps the typed fee. The options list which fees
   came from the stores. A new plan is applied by itself (step 4), and Send to
   Woolworths/Coles starts on that store's cheapest day. Every option still
   prices each item only at a store that sells it. The API also takes
   `?day=<weekday|any>` (`within` wins).

### Fees per day (issue #70)
The Fill trolley bookmark reads the store's delivery times while it reserves
one and reports `delivery_days: [{date, cheapest_fee}]` (≤ 21) with its
report; each report replaces the earlier one for those days. Woolworths: every
open time of `GET /api/v3/ui/fulfilment/windows`. Coles: the open times in the
"Choose delivery slot" drawer for the day it picked. Past days are ignored.

### Modes
| Mode | Order |
|---|---|
| `minimise_total` (default) | buys every item › store takes it and within cap › fewer avoidable dislikes › total › delivery › fewer stores |
| `minimise_delivery` | … › delivery › total › fewer stores |
| `woolworths_only` / `coles_only` | that store's option first, the rest by total |
| `manual` | the current stores first, the rest by total |

### The best mix
Delivery depends on each store's subtotal (free from an amount, a minimum
order, a cap on all fees), so the cheapest store per item is not always the
cheapest order. With **16 or fewer** items that have a choice of store,
every combination is tried (`exact: true`). Above that, a local search from
three starts (cheapest per item, Woolworths where possible, Coles where
possible) moves one item at a time while it helps (`exact: false`; the page
says so). Code: `services/order_plan/search.rs`.

### Delivery fees (Settings › Delivery)
The app cannot read the stores' fees without the household's login
(`docs/FEAT_WOOLWORTHS_ACCESS.md`), so each store's **delivery fee**, **free
delivery from** and **minimum order** are typed once, with the default mode
and **most to spend on delivery** (per order, all stores). Empty = not set.
**Usual delivery day** (a weekday or none) is where the Send to Woolworths
day chooser starts (`FEATURE_TROLLEY_HANDOFF.md`). Woolworths also has a
**Delivery address** (`store_delivery_fees.delivery_address`, ≤ 200 chars,
blank = the account's): one of the account's saved addresses, which the
bookmark picks before a time.
**Allowed delivery days** (weekday toggles; none = any day,
`order_preferences.allowed_delivery_days`): the planner only uses the
stores' fees on these days. When the weekday asked for (or the usual day)
is not allowed, or no allowed day falls within the **Delivery comes** range,
`/order` says so above the options and nothing is **Recommended**, so
nothing is applied by itself. The usual day must be an allowed day (422).
An unset fee counts as $0 and every option using it says so. Saving shows
**Undo**. Amounts are $0–$1000 in whole cents.

### Delivery timing (issue #74)

Settings › Delivery saves an optional **Need by** date and time in
`Australia/Sydney` and an optional freshness interval of 1–168 whole hours.
Set **Perishable** in the grocery item editor for items that need that
interval. Clearing the deadline clears the freshness interval.

With a deadline, the planner uses the cheapest available window for each
store part. The entire window must fit the chosen date range and allowed
weekdays and end by the deadline. If the part contains any marked items,
the window must also start no earlier than the deadline minus the freshness
interval. Both bounds are inclusive. The split planner can schedule other
items earlier at another store. Each option shows its matching windows or
why it could not be scheduled. Options with missing, stale or incompatible
windows cannot be recommended.

The separate **Refresh delivery windows** bookmarks read the logged-in
store's current delivery calendar and replace that store's snapshot via
`POST /api/store-tab/delivery-windows`. The route checks the store-tab shared
secret and matching store origin. A snapshot expires after 15 minutes.
Woolworths reports available regular windows. Coles reports the windows in
its visible week. Refreshes do not reserve slots or fill trolleys. Confirm
the matching window at the store before paying.

The settings API exposes `need_by` (local ISO datetime or null) and
`perishable_hours` (integer or null). Grocery item create, edit and restore
accept `perishable`. A store part on an order option adds `delivery_window`
(`{start, end, fee}` or null) and `delivery_problem` (`windows_unavailable`,
`no_matching_window`, `need_by_passed` or null).


| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/delivery-settings` | Clerk | 200 `{stores: [{store, store_name, delivery_fee, free_delivery_over, minimum_order}], mode, max_delivery_spend, default_delivery_day, allowed_delivery_days}` |
| `PUT` | `/api/delivery-settings` | Clerk | 200 the saved settings; 422 amount out of range, unknown mode or day, usual day not an allowed day. Stores left out keep their rules |
| `GET` | `/api/order-options[?mode=&day=&within=]` | Clerk | 200 `{mode, delivery_day, within_days, store_fees: [{store, store_name, date, fee}], options: [{kind, label, recommended, is_current, stores: [{store, lines, subtotal, delivery_fee, fee_known, free_delivery, below_minimum, minimum_order}], missing: [{grocery_item_id, name, reason}], disliked: [{grocery_item_id, name, store, store_name, disliked_by, avoidable}], items_total, delivery_total, total, complete, fees_known, meets_minimums, within_delivery_cap, picks}], unchosen, exact, max_delivery_spend, allowed_delivery_days, delivery_problem: null|day_not_allowed|no_allowed_day_in_range}`; 422 unknown mode or day, `within` over 7 |
| `PUT` | `/api/order-stores` `{picks}` | Clerk | 204; 422 nothing changed when an item has no choice at its store |

Data: migration `20261003080000_delivery_settings.sql` — `store_delivery_fees`
(one row per store) and `order_preferences` (one row, `id = 1`);
`20261114090000_default_delivery_day.sql` adds `default_delivery_day`;
`20261115100000_store_delivery_day_fees.sql` keeps the stores' fees per day;
`20261116040000_allowed_delivery_days.sql` adds `allowed_delivery_days`;
`20261116100001_delivery_constraints.sql` adds timing preferences, item
perishability, and the window snapshot table.
Code: `services/order_plan/allowed_days.rs`.
Log: `order options planned` (mode, each option's total, recommended, exact),
`delivery settings saved`, `order stores set for items`.

## What the order review does
1. On the grocery list, **Ready to order** commits the list.
2. **Order** in the nav opens `/order`. For every committed item:
   - with a chosen product → the store is asked again, **only the chosen
     store**, at the item's **current** quantity (deals applied);
   - with no product → listed under **No product chosen yet**, with a link
     back to the list.
3. Lines are grouped by store (Woolworths first), each store with a
   **Subtotal**, then a **Total for items**. Delivery fees are not included.
4. A line shows **Up from $X** / **Down from $X** when the shelf price for one
   moved since it was chosen, and **Deal applied** when a multibuy lowers it.
   Lines at one store in the same "Pick any N" offer share it
   (`order_review/pooling.rs`, see `FEATURE_STORE_INTEGRATION.md`).
5. A line that cannot be bought as it is shows **No price** and why. It is not
   counted, and the subtotal and total read **so far**.
6. **Check prices again** re-runs the review. **Send to Woolworths** sits in
   the Woolworths card (`FEATURE_TROLLEY_HANDOFF.md`).

Nothing here picks a product or a store for the household. It only reports
what was chosen and what it costs now (project goal: less marketing-driven
buying).

## Endpoint

| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/order-review` | Clerk session | 200 the review; 401 |

A store that cannot be asked does **not** fail the request: its lines carry
`status: "store_failed"` and a `problem` sentence.

```json
{
  "stores": [{
    "store": "coles", "store_name": "Coles", "subtotal": "8.85", "complete": true,
    "lines": [{
      "grocery_item_id": "…", "item_name": "oat milk", "quantity": 2,
      "product_id": "c-oat-1l", "product_name": "Barista Oat Milk",
      "brand": "Minor Figures", "package_size": "1L", "url": "…",
      "status": "priced", "price": "3.90", "total_price": "6.50",
      "deal_applied": true, "price_change": "same",
      "chosen_price": "3.90", "chosen_total_price": "3.90",
      "chosen_priced_quantity": 1, "problem": null
    }]
  }],
  "unchosen": [{ "grocery_item_id": "…", "name": "bread", "quantity": 2 }],
  "total": "8.85",
  "complete": false
}
```

| `status` | Meaning | Counted |
|---|---|---|
| `priced` | still sold; today's price | yes (if the store shows a price) |
| `unavailable` | listed but cannot be delivered now | no |
| `not_offered` | no longer in the store's results for the item and its chips | no |
| `store_failed` | the store could not be asked | no |

`price_change`: `same` · `up` · `down` · `null` (a price unknown).
`complete`: every item has a product **and** every line has a price.
Money is a decimal string.

## Rules
- **Only committed items.** Active items are still being reviewed on the list.
- **Current quantity.** The choice's `total_price` was for
  `priced_quantity`; the review prices the item's quantity now.
- **One store per item, one item at a time.** Items are re-priced in turn,
  each only at its chosen store, so the stores see a household's pace. Most
  answers come from the 10-minute search cache.
- **Send count is honest.** The Send button counts active and committed items
  chosen at Woolworths, because that is what a handoff sends
  (`lib/chosen-at-store.ts`, shared with the list page).

## Code
- Planner: `routes/order_options.rs`, `routes/delivery_settings.rs`;
  `services/order_plan/` (`mod.rs` service, `option.rs`, `fees.rs`,
  `search.rs`, `rank.rs`, `build.rs`, `log.rs`); `services/delivery_settings/`;
  `models/delivery_rows.rs`; web `components/order-options/`,
  `components/delivery/`, `routes/settings/delivery.tsx`,
  `lib/api/order-options.ts`, `lib/api/delivery-settings.ts`,
  `lib/order-modes.ts`, `lib/option-notes.ts`, `lib/delivery-form.ts`,
  `hooks/useOrderStores.ts`.
- Backend: `routes/order_review.rs`; `services/order_review/` — `mod.rs`
  (service), `line/` (pure re-pricing of one choice; `tests.rs`,
  `fixtures.rs`), `summary.rs` (pure grouping and totals), `log.rs`;
  `models/schemas/order_review.rs`; `ProductSearchService::search_at` (one
  store for one item).
- Frontend: `routes/order.tsx`; `components/order/` — `OrderReview`
  compound (`.Empty`, `.Unchosen`, `.Store`, `.Total`), `StoreOrderCard`,
  `OrderLineRow`, `PriceChangeBadge`; `lib/api/order-review.ts`,
  `lib/price-change.ts`, `lib/chosen-at-store.ts`.

## Logs
`order reviewed` (stores, lines, unchosen, problems, total, complete) on every
review; `order line needs attention` (warn) for a line with a problem;
`chosen product's price changed` when a shelf price moved.

## Not built (spec "Order Optimisation")
- Fees for a day nobody has filled a trolley since: the stores' times are
  read only by the bookmark, in the household's logged-in tab, and the
  15-minute hold is the store's.
- "Prefer specials" mode.
- Skipping disliked products, and the warning before confirming. The rule
  is ready to call: `services/dislikes/skip.rs` (`FEATURE_DISLIKES.md`).

## Tests
- `backend/src/services/order_review/line/tests.rs`, `summary.rs` — every
  line outcome, grouping, totals.
- `backend/tests/order_review.rs` — through the real router over the fake
  stores: empty, grouped, current quantity, unchosen, price change, product
  gone, store down, auth.
- `frontend/src/components/order/__tests__/`, `lib/__tests__/price-change.test.ts`,
  `lib/__tests__/chosen-at-store.test.ts`.
- `frontend/e2e/order-review.spec.ts` — desktop and phone.
- Planner: `services/order_plan/fees.rs`, `rank.rs`, `tests.rs`,
  `tests_modes.rs` (free delivery reached by moving an item, minimums, the
  cap, every mode, 20 items by local search); `backend/tests/order_options.rs`,
  `delivery_settings.rs`, `delivery_constraints.rs`, `item_selections_per_store.rs`.
- Web: `components/order-options/__tests__`, `components/delivery/__tests__`,
  `lib/__tests__/delivery-form.test.ts`, `option-notes.test.ts`;
  `frontend/e2e/order-options.spec.ts`.
