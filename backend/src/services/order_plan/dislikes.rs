//! Which of an item's chosen products the household dislikes, per store.
//!
//! Pure. The planner respects every member's dislikes
//! ([`DislikeScope::Household`]); a "buy it this time" override on the item
//! sets a dislike aside (`docs/features/FEATURE_DISLIKES.md`).

use super::option::OfferDislike;
use crate::models::db::ItemSelection;
use crate::services::dislikes::skip::{DislikeBook, DislikeScope};
use crate::services::stores::Store;

/// The stores where one of the item's `choices` is disliked, and by whom.
/// `book` must hold the overrides of that one item.
pub fn offer_dislikes(book: &DislikeBook, choices: &[ItemSelection]) -> Vec<OfferDislike> {
    Store::ALL
        .into_iter()
        .filter_map(|store| {
            let mut disliked_by: Vec<String> = Vec::new();
            for choice in choices.iter().filter(|choice| choice.store == store) {
                if !book.skips(store, &choice.product_id, DislikeScope::Household) {
                    continue;
                }
                for name in book.disliked_by(store, &choice.product_id, DislikeScope::Household) {
                    if !disliked_by.iter().any(|known| known == name) {
                        disliked_by.push(name.to_string());
                    }
                }
            }
            (!disliked_by.is_empty()).then_some(OfferDislike { store, disliked_by })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use super::*;
    use crate::models::db::{DislikeOverride, GroceryItem, ProductDislike};
    use crate::services::order_review::fixtures::{choice, item};
    use crate::services::stores::Store::{Coles, Woolworths};

    fn dislike(name: &str, store: Store, product_id: &str) -> ProductDislike {
        ProductDislike {
            id: Uuid::new_v4(),
            user_id: format!("user_{name}"),
            user_name: name.into(),
            store,
            product_id: product_id.into(),
            product_name: "Milk".into(),
            brand: None,
            package_size: None,
            disliked_at: Utc::now(),
        }
    }

    fn set_aside(item: &GroceryItem, store: Store, product_id: &str) -> DislikeOverride {
        DislikeOverride {
            id: Uuid::new_v4(),
            grocery_item_id: item.id,
            store,
            product_id: product_id.into(),
            overridden_by: "user_jesse".into(),
            overridden_at: Utc::now(),
        }
    }

    #[test]
    fn names_each_member_once_for_the_store_they_dislike() {
        let milk = item("milk", 1);
        let choices = [
            choice(&milk, Coles, "c-1", dec!(3)),
            choice(&milk, Coles, "c-2", dec!(4)),
            choice(&milk, Woolworths, "w-1", dec!(5)),
        ];
        let book = DislikeBook::new(
            vec![
                dislike("phu", Coles, "c-1"),
                dislike("phu", Coles, "c-2"),
                dislike("jesse", Coles, "c-2"),
            ],
            vec![],
        );

        assert_eq!(
            offer_dislikes(&book, &choices),
            vec![OfferDislike {
                store: Coles,
                disliked_by: vec!["phu".into(), "jesse".into()],
            }]
        );
    }

    #[test]
    fn a_dislike_set_aside_for_the_item_does_not_count() {
        let milk = item("milk", 1);
        let choices = [choice(&milk, Coles, "c-1", dec!(3))];
        let book = DislikeBook::new(
            vec![dislike("phu", Coles, "c-1")],
            vec![set_aside(&milk, Coles, "c-1")],
        );

        assert!(offer_dislikes(&book, &choices).is_empty());
    }
}
