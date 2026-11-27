# Feature: Purchase History

Status: **implemented** for Woolworths and Coles handoffs. Every product
reported as added by the store tab is saved as bought, with its price, so the
product picker can show past purchases and Order › History can show spending
(`FEATURE_SPENDING_ANALYSIS.md`). Steps for people: `docs/using-the-app.md`
("Do a Woolworths shop", "Undo a shop saved by mistake") — keep those in
step with this spec.

## When a shop counts as bought
The app cannot see payment. Jesse chose (2026-10-02): **a shop is bought
when the store tab reports a filled trolley.** He wants this as automatic as
possible, and **every save must be easy to undo**. Importing the stores'
own order history later is the goal; `purchase_orders.source` leaves room
for it (only `trolley_fill` today).

```
bookmark report (routes/store_tab.rs) ─▶ 200 to the bookmark at once
             │ tokio::spawn — purchases::record_in_background
             ▼
  lines "added" whose item is still active/committed
  ─▶ priced (fresh store search, else the saved choice)
  ─▶ delivery fee split across lines
  ─▶ one transaction: product rows (category set) + order + purchases,
     items → ordered
```

## Rules
- **Once per handoff:** `trolley_handoff_id` is UNIQUE; a second save is a
  no-op (`ON CONFLICT DO NOTHING`).
- **Nothing added, nothing saved.** Failed lines stay on the list.
- **Price:** `search_at` on the store for the product id at fill time; if
  the search fails or misses it, the saved choice's price. `unit_price` is
  what was paid each, `shelf_price` the price before any deal.
- **Delivery fee** (from the reserved window) is split across lines by line
  cost, in whole cents; the last line takes the remainder.
- **Items:** bought items move to `ordered` and leave the list.
  `item_status_before` keeps `active` or `committed`.
- **Undo** (`DELETE /api/purchase-orders/{id}`) deletes the order and its
  purchases and puts each item back to `item_status_before`, only if it is
  still `ordered`. A deleted item stays deleted. A second Undo is a 404.
  Undo never touches the store's trolley. Each of the shop's products is
  then refreshed (see **Bought products**).
- **Category** (one per product, kept on its **Bought products** row;
  worked out from the newest purchase's texts; re-sorted only on request):
  1. the store's own category, mapped to ours, when it gives one;
  2. else keywords on the item and product name: whole words, plural -s/-es,
     longest keyword wins, ties go to the category listed first;
  3. else **Other**. `category_source` says which: `store`, `name`, `none`.
  Categories: Fruit & veg, Meat & seafood, Dairy & eggs, Bakery, Frozen,
  Drinks, Pantry, Snacks & sweets, Household, Health & beauty, Baby, Pet.
- **Bought products** (`bought_products`, one row per `(store,
  product_id)`, issue #91). Owned by the purchase history
  (`services/purchases/products.rs`); nothing else writes it. Refresh:
  1. saving a purchase writes its product's name, brand, size and store
     category, and works out its category, unless a newer purchase is saved
     already (`refreshed_at`);
  2. Undo refreshes each of the shop's products from its newest purchase
     left, and deletes the row when none is left;
  3. **Sort categories again** recomputes every row's category and returns
     how many products changed.
  Only this table holds a category; every `Purchase` read joins it. Each
  purchase keeps its own copy of the name, brand, size and store category
  as a snapshot of that day; totals only ever come from purchases. A foreign
  key from `purchases (store, product_id)` means every purchase has its row.
- **No suggestions.** History is shown as facts (times bought, prices paid);
  nothing ranks or recommends a product by it.

## Endpoints (all Clerk)

| Method | Path | Answer |
|---|---|---|
| `GET` | `/api/purchase-orders` | the 20 newest shops, with product counts |
| `DELETE` | `/api/purchase-orders/{id}` | `{items_restored}`; 404 if not saved |
| `POST` | `/api/purchase-history/products` `{products:[{store, product_id}]}` | per product bought before: `times_bought`, `purchases` (newest first) |
| `POST` | `/api/purchase-history/recategorise` | `{changed}` — products whose category changed under today's rules |

## Data
Migration `20261002120000_purchase_history` (timestamp version so parallel
branches cannot clash): `purchase_orders` (store, items_total,
delivery_fee, recorded_by, source, trolley_handoff_id, bought_at) and
`purchases` (order_id CASCADE, grocery_item_id SET NULL, item_name,
item_key, item_status_before, store, product_id, product_name, brand,
package_size, quantity, unit_price, shelf_price, total_price,
delivery_fee_share, store_category, category, category_source, bought_at).
Migration `20261119090000_bought_products`: `bought_products` (store,
product_id, product_name, brand, package_size, store_category, category,
category_source, refreshed_at), backfilled from each product's newest
purchase with that purchase's category, then the foreign key from
`purchases`. Migration `20261127090000_drop_purchase_category` drops
`purchases.category` and `category_source`.

## Code
- Backend: `services/purchases/` (`mod.rs` service + `record`,
  `record.rs`, `price.rs`, `fee.rs`, `category.rs` + `category/keywords.rs`,
  `history.rs`, `undo.rs`, `products.rs` bought products,
  `repository.rs` writes, `read_repository.rs` reads,
  `product_repository.rs` bought products SQL, `log.rs`), `models/purchase_rows.rs`, `routes/purchases.rs`,
  `models/schemas/purchases.rs`; `grocery/repository.rs` `mark_ordered`,
  `unmark_ordered`.
- Frontend: `lib/api/purchases.ts`, `hooks/useProductHistory.ts`,
  `useSavedPurchase.ts`, `useUndoPurchase.ts`,
  `components/products/past-purchases.tsx` ("Bought N times" in the
  comparison), `components/trolley/saved-purchase.tsx` (Undo in the sheet).

## Not built
- Importing past orders from Woolworths or Coles
  ([issue #78](https://github.com/jesseross42/smart-home-groceries-listing/issues/78)).

## Tests
- `backend/tests/purchase_history.rs` — save after fill, once per handoff,
  nothing added, Undo (twice, deleted item), picker lookup, recategorise,
  auth. Saving is in the background: tests poll
  (`tests/common/purchases.rs` `orders_once_saved`).
- `backend/tests/bought_products.rs` — record kept on save, Undo removes
  or refreshes it, an older purchase never overwrites a newer one.
  `bought_products_backfill.rs` — the migration on existing purchases.
- Unit tests in `price/tests.rs`, `fee.rs`, `category.rs`, `history.rs`.
- `frontend/src/components/products/__tests__/past-purchases.test.tsx`,
  `components/trolley/__tests__/saved-purchase.test.tsx`.
