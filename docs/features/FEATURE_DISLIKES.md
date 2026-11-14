# Feature: Disliked Products

Status: **implemented** for manual choice and for the order planner
(`services/order_plan`, household scope). Spec: "Purchase History and
Preferences" › "Disliked items" and "Per-user vs household preferences".

How a person uses it: [`using-the-app.md` › Product dislikes](../using-the-app.md#product-dislikes).

## What this feature does
- **Per member.** Each household member has their own dislikes. A dislike
  is one store product (`store` + the store's `product_id`).
- **Household view.** Everyone sees everyone's dislikes:
  - on the Select item page, a disliked product shows *"Phu disliked this
    item previously."* (*"You …"* for your own);
  - on the grocery list and on `/order`, a chosen product that somebody
    dislikes shows the same sentence, and *"OK to buy it this time."* when
    the dislike is set aside for the item. Read-only: the dislike is changed
    on the Select item page;
  - **Dislikes** (`/settings/dislikes`) lists every member's dislikes, yours
    first.
- **Manual choice is never blocked.** A disliked product keeps its
  **Choose** button. The warning is all that changes.
- **Override for this order.** **Buy it this time** on the warning sets the
  dislike aside for that one list item. The dislike itself stays. **Undo**
  brings it back.
- **Override for good.** **Remove my dislike** (in the comparison or on the
  Dislikes page) deletes your own dislike. Nobody can remove another
  member's dislike.
- **Nothing is suggested.** A dislike only warns or skips; it never offers
  another product instead (project goal: less marketing-driven buying).

## Rules for the order planner

The planner (`FEATURE_ORDER_OPTIMISATION.md`) only moves an item between the
products the household chose, one per store. It uses
`DislikeScope::Household`: any member's dislike counts.

- **Skip.** The best mix does not buy an item at a store whose chosen product
  is disliked when another store's chosen product is not.
- **Override.** *Buy it this time* on the item sets the dislike aside, so the
  planner may pick that product again.
- **Every product disliked.** The best-ranked store is kept. The option says
  who disliked it (`disliked[].avoidable = false`).
- **Other options.** *All at Woolworths*, *All at Coles* and *Your current
  stores* still buy what the household chose. Each names the disliked
  products it buys, and ranks after an option that avoids them (after
  "buys every item" and "the store takes it", before price).

Code: `services/order_plan/dislikes.rs` (who dislikes which store's choice),
`option.rs` (`Candidate::allowed_prices`, `DislikedPick`), `rank.rs`.
`services/dislikes/skip.rs` keeps `DislikeScope::Member` and
`pick_for_automated_order` for a planner that ranks products itself.

## Endpoints

| Method | Path | Answer |
|---|---|---|
| `GET` | `/api/product-dislikes` | 200, every member's dislikes, newest first |
| `PUT` | `/api/product-dislikes` | 200 the saved dislike (also when it existed) |
| `DELETE` | `/api/product-dislikes/{store}/{product_id}` | 204 your own dislike removed (also when none) |
| `GET` | `/api/dislike-overrides` | 200, every override on the list |
| `PUT` | `/api/grocery-items/{id}/dislike-override` | 200 the override |
| `DELETE` | `/api/grocery-items/{id}/dislike-override/{store}/{product_id}` | 204 (also when none) |

All need a Clerk session (401 without).

`PUT /api/product-dislikes` body:
`{ "store", "product_id", "name", "brand"?, "package_size"? }`. The name,
brand and size are only a label for the Dislikes page, as the comparison
showed them. 422: unknown store, empty id or name, or too long.

A dislike:

```json
{
  "store": "coles", "store_name": "Coles", "product_id": "c-milk-3l",
  "name": "Full Cream Milk", "brand": "Coles", "package_size": "3L",
  "user_id": "user_…", "user_name": "phu", "mine": false,
  "disliked_at": "2026-10-02T05:00:00Z"
}
```

`PUT …/dislike-override` body: `{ "store", "product_id" }`. 404 unknown
item; 409 the item is ordered; 422 nobody dislikes that product.

## The member's name
The session token has an id and maybe an email, not a display name.
`user_name` is the part of the email before `@`, or *"A household member"*
when there is no email (`services/dislikes/member_name.rs`). It is saved
with the dislike. When Clerk sends a real name, change only that file.

## Data
Migration `20261002140000_product_dislikes.sql`:
- `product_dislikes` — one row per member per product
  (`UNIQUE (user_id, store, product_id)`). Disliking again keeps the date
  and refreshes the label.
- `dislike_overrides` — one row per item per product, `ON DELETE CASCADE`
  from `grocery_items`. An override lives as long as the item: the next
  time the item is added it is a new row, so the dislike counts again.

## Code
- Backend: `routes/dislikes.rs`; `services/dislikes/` — `mod.rs` (rules),
  `skip.rs` (pure, for the optimiser; tests in `skip_tests.rs`),
  `member_name.rs`, `repository.rs`, `overrides_repository.rs`, `log.rs`;
  `models/schemas/dislikes.rs`; `models/dislike_rows.rs`.
- Web app: `lib/api/dislikes.ts`, `lib/dislikes.ts` (pure: who dislikes it,
  the warning sentence, grouping); `hooks/useItemDislikes.ts`;
  `components/dislikes/` — `product-dislikes.tsx` (context scoped to one
  `<ProductResults>`), `dislike-button.tsx`, `dislike-notice.tsx`,
  `household-dislikes.tsx`; `routes/settings/dislikes.tsx`. The list and
  `/order`: `lib/dislike-marks.ts` (read by both route loaders; a failed read
  shows no marks) and `components/dislikes/dislike-mark.tsx` (provider + mark,
  used by `selections/chosen-product.tsx` and `order/order-line-row.tsx`).

## Logs
`product disliked`, `product dislike removed`, `dislike overridden for this
order`, `dislike override cleared`, `dislike override refused` (warn).

## Not built yet
- The optimiser itself, and so the "warn before the order is confirmed"
  screen (`FEATURE_ORDER_OPTIMISATION.md`).

Track planner integration in [issue #76](https://github.com/jesseross42/smart-home-groceries-listing/issues/76).

## Tests
- `backend/src/services/dislikes/skip_tests.rs` — every scope and override
  rule, and the all-disliked fallback.
- `backend/tests/product_dislikes.rs`, `backend/tests/dislike_overrides.rs` —
  the routes, the household view with another member, and the book.
- `frontend/src/lib/__tests__/dislikes.test.ts`,
  `frontend/src/components/dislikes/__tests__/`.
- `frontend/e2e/dislikes.spec.ts` — desktop and phone.
