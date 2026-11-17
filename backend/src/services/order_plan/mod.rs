//! The order planner: the ways to buy the committed list, with delivery,
//! ranked the household's way.
//!
//! Every committed item's chosen products are re-priced today at their
//! stores (several products at one store joined into one line, [`combine`]), the same way the order review does. The
//! planner then costs four ways to buy them — all at Woolworths, all at
//! Coles, the best mix, and the stores the order buys from now — adding each
//! store's delivery fee from Settings › Delivery, and ranks them by the mode
//! (`docs/features/FEATURE_ORDER_OPTIMISATION.md`).
//!
//! The planner never picks a product: it only moves an item between products
//! the household chose. A store whose chosen product somebody dislikes is
//! skipped for that item when another store has one nobody dislikes
//! ([`dislikes`]); a disliked product an option still buys is named on it. Using an option is `PUT /api/order-stores`
//! (`SelectionService::buy_at`).
//!
//! Each store's fee is the cheapest it showed on the delivery day when the
//! bookmark reported one ([`fee_days`], issue #70), else the typed fee. Only
//! the household's allowed delivery days count, and nothing is recommended
//! when the delivery asked for cannot come on one ([`allowed_days`], #74). A
//! deadline uses fresh store windows and accounts for perishable items ([`timing`]).
//!
//! Delivery charge in [`fees`]; one option in [`option`]; the best mix in
//! [`search`]; ranking in [`rank`]; putting it together in [`build`].

mod allowed_days;
mod build;
mod combine;
mod dislikes;
mod fee_days;
mod fees;
mod log;
mod option;
mod rank;
mod search;
mod timing;

pub use allowed_days::DayProblem;
pub use fees::DeliveryCharge;
pub use option::{DislikedPick, Missing, OptionKind, PlanOption, StorePart};
pub use timing::{StoreTiming, TimingProblem};

use std::collections::HashMap;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemStatus, ItemSelection};
use crate::models::delivery_rows::{DeliveryDay, OrderMode};
use crate::services::delivery_day_fees::{store_today, within, DayFee, DeliveryDayFeeService};
use crate::services::delivery_settings::{DeliverySettings, DeliverySettingsService};
use crate::services::dislikes::skip::DislikeBook;
use crate::services::dislikes::DislikeService;
use crate::services::grocery::repository as items;
use crate::services::order_review::reprice;
use crate::services::product_search::ProductSearchService;
use crate::services::selections::parts::parts;
use crate::services::selections::repository as selections;
use crate::services::stores::{Store, StoreClients};
use option::Candidate;

/// The order's options, best first.
#[derive(Debug, Clone)]
pub struct OrderPlan {
    pub mode: OrderMode,
    pub options: Vec<PlanOption>,
    /// The index of the option to use, when the best one can be placed as it
    /// is (every item bought, store minimums met, within the delivery cap).
    pub recommended: Option<usize>,
    /// The index of the option the order buys from now, if it is listed.
    pub current: Option<usize>,
    /// Committed items with no product chosen at any store.
    pub unchosen: Vec<GroceryItem>,
    /// The best mix was proven best (every combination tried).
    pub exact: bool,
}

/// A plan, the settings it used, and the delivery day it was costed for.
#[derive(Debug, Clone)]
pub struct PlannedOrder {
    pub plan: OrderPlan,
    /// The settings, with the stores' own fees swapped in.
    pub settings: DeliverySettings,
    /// The day asked for, else the usual one; `None` = the cheapest day.
    pub delivery_day: Option<DeliveryDay>,
    /// Days ahead the delivery must come within, when that was asked.
    pub within_days: Option<u8>,
    /// The stores' own fees the plan used instead of the typed ones.
    pub store_fees: Vec<DayFee>,
    /// Why the delivery asked for cannot come on an allowed day, if so.
    pub day_problem: Option<DayProblem>,
}

/// Which day the plan costs delivery on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DayChoice {
    /// The usual day from Settings › Delivery (each store's cheapest day
    /// when none is set).
    #[default]
    Usual,
    /// Each store's cheapest reported day.
    Any,
    /// This weekday.
    On(DeliveryDay),
    /// Each store's cheapest day from today up to this many days ahead
    /// (0 = today only, at most 7).
    Within(u8),
}

/// Plans the order.
pub struct OrderPlanService<'a> {
    pool: &'a PgPool,
    stores: &'a StoreClients,
}

impl<'a> OrderPlanService<'a> {
    /// Borrows the pool and the store clients for one request.
    pub fn new(pool: &'a PgPool, stores: &'a StoreClients) -> Self {
        Self { pool, stores }
    }

    /// The committed list's options under `mode`, or the saved mode, with
    /// delivery costed on `day`.
    ///
    /// Items are re-priced one store at a time, as the order review does;
    /// most answers come from the 10-minute search cache.
    pub async fn plan(
        &self,
        mode: Option<OrderMode>,
        day: DayChoice,
    ) -> Result<PlannedOrder, ApiError> {
        let mut settings = DeliverySettingsService::new(self.pool).read().await?;
        let mode = mode.unwrap_or(settings.mode);
        let delivery_day = match day {
            DayChoice::Usual => settings.default_delivery_day,
            DayChoice::On(day) => Some(day),
            DayChoice::Any | DayChoice::Within(_) => None,
        };
        let within_days = match day {
            DayChoice::Within(days_ahead) => Some(days_ahead),
            _ => None,
        };
        let allowed = &settings.allowed_delivery_days;
        let day_problem = allowed_days::problem(allowed, delivery_day, store_today(), within_days);
        let store_fees = if settings.need_by.is_some() {
            Vec::new()
        } else {
            let mut reported = allowed_days::only_allowed(
                &DeliveryDayFeeService::new(self.pool).upcoming().await?,
                allowed,
            );
            if let Some(days_ahead) = within_days {
                reported = within(&reported, store_today(), days_ahead);
            }
            fee_days::use_store_fees(&mut settings, &reported, delivery_day)
        };
        let committed = items::list_items(self.pool)
            .await?
            .into_iter()
            .filter(|item| item.status == GroceryItemStatus::Committed);
        let mut choices: HashMap<Uuid, Vec<ItemSelection>> = HashMap::new();
        for choice in selections::list_every(self.pool).await? {
            choices
                .entry(choice.grocery_item_id)
                .or_default()
                .push(choice);
        }
        let search = ProductSearchService::new(self.pool, self.stores);
        let dislike_service = DislikeService::new(self.pool);
        let household_dislikes = dislike_service.list_household().await?;
        let mut overrides: HashMap<Uuid, Vec<_>> = HashMap::new();
        for set_aside in dislike_service.list_overrides().await? {
            overrides
                .entry(set_aside.grocery_item_id)
                .or_default()
                .push(set_aside);
        }

        let mut candidates = Vec::new();
        let mut unchosen = Vec::new();
        for item in committed {
            let Some(item_choices) = choices.remove(&item.id) else {
                unchosen.push(item);
                continue;
            };
            let Some(current) = item_choices.iter().find(|c| c.for_order).map(|c| c.store) else {
                unchosen.push(item);
                continue;
            };
            let book = DislikeBook::new(
                household_dislikes.clone(),
                overrides.remove(&item.id).unwrap_or_default(),
            );
            let dislikes = dislikes::offer_dislikes(&book, &item_choices);
            let mut offers = Vec::new();
            for store in Store::ALL {
                let at_store: Vec<_> = item_choices
                    .iter()
                    .filter(|c| c.store == store)
                    .cloned()
                    .collect();
                let mut lines = Vec::new();
                for (part, choice) in parts(&item, at_store) {
                    let outcome = search.search_at(&part, store).await;
                    lines.push(reprice(part, choice, outcome));
                }
                offers.extend(combine::combine(&item, lines));
            }
            candidates.push(Candidate {
                item,
                current,
                offers,
                dislikes,
            });
        }

        let timing =
            timing::Timing::load(self.pool, &settings, delivery_day, within_days, Utc::now())
                .await?;
        let mut plan = build::plan(mode, &candidates, unchosen, &settings, &timing);
        if day_problem.is_some() {
            // No option can be delivered as asked, so none is recommended.
            plan.recommended = None;
        }
        log::planned(&plan);
        Ok(PlannedOrder {
            plan,
            settings,
            delivery_day,
            within_days,
            store_fees,
            day_problem,
        })
    }
}

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_dislikes;
#[cfg(test)]
mod tests_modes;
