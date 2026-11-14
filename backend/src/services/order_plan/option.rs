//! One way to buy the order: which store each item comes from, and what that
//! costs with delivery.
//!
//! Pure: the re-priced choices and the household's fee rules in, a costed
//! [`PlanOption`] out.

use rust_decimal::Decimal;
use uuid::Uuid;

use super::fees::{charge, DeliveryCharge};
use crate::models::db::GroceryItem;
use crate::services::delivery_settings::DeliverySettings;
use crate::services::order_review::OrderLine;
use crate::services::stores::Store;

/// A committed item, and each of its chosen products re-priced today.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub item: GroceryItem,
    /// The store the order buys the item from now (`for_order`).
    pub current: Store,
    /// One line per store the household chose a product at.
    pub offers: Vec<OrderLine>,
    /// The stores whose chosen product somebody dislikes, and who.
    pub dislikes: Vec<OfferDislike>,
}

/// Who dislikes the product an item would be bought as at one store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferDislike {
    pub store: Store,
    /// Member names, never empty.
    pub disliked_by: Vec<String>,
}

impl Candidate {
    /// The item's line at `store`, priced or not.
    pub fn offer_at(&self, store: Store) -> Option<&OrderLine> {
        self.offers.iter().find(|line| line.choice.store == store)
    }

    /// What the item costs at `store` today, when it can be bought there.
    pub fn price_at(&self, store: Store) -> Option<Decimal> {
        self.offer_at(store).and_then(OrderLine::counted_total)
    }

    /// The members who dislike the item's product at `store`; empty for none.
    pub fn disliked_by(&self, store: Store) -> &[String] {
        self.dislikes
            .iter()
            .find(|dislike| dislike.store == store)
            .map_or(&[], |dislike| dislike.disliked_by.as_slice())
    }

    /// The stores an automated choice may buy the item at, with today's
    /// price: the ones nobody dislikes, or every priced store when all of
    /// them are disliked.
    pub fn allowed_prices(&self) -> Vec<(Store, Decimal)> {
        let priced: Vec<(Store, Decimal)> = Store::ALL
            .into_iter()
            .filter_map(|store| self.price_at(store).map(|price| (store, price)))
            .collect();
        let liked: Vec<(Store, Decimal)> = priced
            .iter()
            .copied()
            .filter(|(store, _)| self.disliked_by(*store).is_empty())
            .collect();
        if liked.is_empty() {
            priced
        } else {
            liked
        }
    }

    /// Buying at `store` buys a disliked product although another store has
    /// one nobody dislikes.
    pub fn avoidable_dislike(&self, store: Store) -> bool {
        !self.disliked_by(store).is_empty()
            && self
                .allowed_prices()
                .iter()
                .any(|(allowed, _)| self.disliked_by(*allowed).is_empty())
    }
}

/// Which way of buying an option is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    /// The cheapest mix of stores the planner found.
    Split,
    /// Everything that can be, from one store.
    Only(Store),
    /// The stores the order buys from now.
    Current,
}

/// What an option buys at one store.
#[derive(Debug, Clone)]
pub struct StorePart {
    pub store: Store,
    pub lines: Vec<OrderLine>,
    pub subtotal: Decimal,
    pub charge: DeliveryCharge,
    pub minimum_order: Option<Decimal>,
}

/// An item an option cannot buy, and why.
#[derive(Debug, Clone)]
pub struct Missing {
    pub item: GroceryItem,
    pub reason: String,
}

/// A disliked product an option buys.
#[derive(Debug, Clone)]
pub struct DislikedPick {
    pub item: GroceryItem,
    pub store: Store,
    pub disliked_by: Vec<String>,
    /// Another store has a product for the item that nobody dislikes.
    pub avoidable: bool,
}

/// One costed way to buy the order.
#[derive(Debug, Clone)]
pub struct PlanOption {
    pub kind: OptionKind,
    /// Stores with something to buy, in display order.
    pub parts: Vec<StorePart>,
    pub missing: Vec<Missing>,
    /// The disliked products it buys, to show before the order goes on.
    pub disliked: Vec<DislikedPick>,
    pub items_total: Decimal,
    pub delivery_total: Decimal,
    pub total: Decimal,
    /// Delivery fees stay within the household's cap (or there is none).
    pub within_cap: bool,
}

impl PlanOption {
    /// Every item is bought.
    pub fn complete(&self) -> bool {
        self.missing.is_empty()
    }

    /// Every store used has a fee set, so the delivery total is real.
    pub fn fees_known(&self) -> bool {
        self.parts.iter().all(|part| part.charge.fee_known)
    }

    /// No store's part is under that store's minimum order.
    pub fn meets_minimums(&self) -> bool {
        self.parts.iter().all(|part| !part.charge.below_minimum)
    }

    /// The option can be placed and respects the household's cap.
    pub fn acceptable(&self) -> bool {
        self.meets_minimums() && self.within_cap
    }

    /// How many disliked products it buys that another store would avoid.
    pub fn avoidable_dislikes(&self) -> usize {
        self.disliked.iter().filter(|pick| pick.avoidable).count()
    }

    /// Each bought item and its store.
    pub fn picks(&self) -> Vec<(Uuid, Store)> {
        self.parts
            .iter()
            .flat_map(|part| part.lines.iter().map(|line| (line.item.id, part.store)))
            .collect()
    }
}

/// Costs buying `candidates[i]` at `assignment[i]`; `None` leaves it out.
/// An assigned store must price the item (the caller checks).
pub fn evaluate(
    kind: OptionKind,
    candidates: &[Candidate],
    assignment: &[Option<Store>],
    settings: &DeliverySettings,
) -> PlanOption {
    let mut missing = Vec::new();
    let mut parts: Vec<StorePart> = Vec::new();
    for store in Store::ALL {
        let lines: Vec<OrderLine> = candidates
            .iter()
            .zip(assignment)
            .filter(|(_, at)| **at == Some(store))
            .filter_map(|(candidate, _)| candidate.offer_at(store).cloned())
            .collect();
        if lines.is_empty() {
            continue;
        }
        let subtotal = lines.iter().filter_map(OrderLine::counted_total).sum();
        let rules = settings.rules_for(store);
        parts.push(StorePart {
            store,
            lines,
            subtotal,
            charge: charge(&rules, subtotal),
            minimum_order: rules.minimum_order,
        });
    }
    let mut disliked = Vec::new();
    for (candidate, at) in candidates.iter().zip(assignment) {
        match at {
            None => missing.push(Missing {
                item: candidate.item.clone(),
                reason: missing_reason(kind, candidate),
            }),
            Some(store) if !candidate.disliked_by(*store).is_empty() => {
                disliked.push(DislikedPick {
                    item: candidate.item.clone(),
                    store: *store,
                    disliked_by: candidate.disliked_by(*store).to_vec(),
                    avoidable: candidate.avoidable_dislike(*store),
                });
            }
            Some(_) => {}
        }
    }
    let items_total: Decimal = parts.iter().map(|part| part.subtotal).sum();
    let delivery_total: Decimal = parts.iter().map(|part| part.charge.fee).sum();
    PlanOption {
        kind,
        parts,
        missing,
        disliked,
        items_total,
        delivery_total,
        total: items_total + delivery_total,
        within_cap: settings
            .max_delivery_spend
            .is_none_or(|cap| delivery_total <= cap),
    }
}

/// Why `kind` cannot buy the item.
fn missing_reason(kind: OptionKind, candidate: &Candidate) -> String {
    match kind {
        OptionKind::Only(store) => match candidate.offer_at(store) {
            Some(line) => line
                .problem
                .clone()
                .unwrap_or_else(|| format!("No price at {} today.", store.display_name())),
            None => format!("No product chosen at {}.", store.display_name()),
        },
        OptionKind::Split | OptionKind::Current => candidate
            .offer_at(candidate.current)
            .and_then(|line| line.problem.clone())
            .unwrap_or_else(|| "No price at any store today.".into()),
    }
}
