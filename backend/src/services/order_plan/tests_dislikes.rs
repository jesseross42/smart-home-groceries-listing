//! The planner and the household's dislikes. Pure.

use rust_decimal_macros::dec;

use super::build::plan;
use super::option::OptionKind;
use super::test_fixtures::{candidate, disliked, settings};
use crate::models::delivery_rows::OrderMode;
use crate::services::stores::Store::{Coles, Woolworths};

#[test]
fn the_best_mix_skips_a_disliked_product_when_another_store_has_one() {
    let items = [disliked(
        candidate("milk", Coles, &[(Woolworths, dec!(9)), (Coles, dec!(3))]),
        Coles,
        "phu",
    )];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(0), None, None),
    );

    let best = &planned.options[0];
    assert_eq!(best.picks(), vec![(items[0].item.id, Woolworths)]);
    assert!(best.disliked.is_empty());
    assert_eq!(planned.recommended, Some(0));
}

#[test]
fn an_option_that_buys_an_avoidable_dislike_says_who_disliked_it() {
    let items = [disliked(
        candidate("milk", Coles, &[(Woolworths, dec!(9)), (Coles, dec!(3))]),
        Coles,
        "phu",
    )];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(0), None, None),
    );

    let coles = planned
        .options
        .iter()
        .find(|option| option.kind == OptionKind::Only(Coles))
        .expect("the Coles option is still offered");
    assert_eq!(coles.disliked.len(), 1);
    assert_eq!(coles.disliked[0].disliked_by, vec!["phu".to_string()]);
    assert!(coles.disliked[0].avoidable);
}

#[test]
fn when_every_store_is_disliked_the_cheapest_is_kept_with_a_warning() {
    let milk = candidate("milk", Coles, &[(Woolworths, dec!(9)), (Coles, dec!(3))]);
    let items = [disliked(disliked(milk, Coles, "phu"), Woolworths, "jesse")];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(0), None, None),
    );

    let best = &planned.options[0];
    assert_eq!(best.picks(), vec![(items[0].item.id, Coles)]);
    assert_eq!(best.disliked[0].disliked_by, vec!["phu".to_string()]);
    assert!(!best.disliked[0].avoidable);
    assert_eq!(planned.recommended, Some(0));
}

#[test]
fn a_single_store_mode_keeps_its_store_first_and_still_warns() {
    let items = [disliked(
        candidate("milk", Coles, &[(Woolworths, dec!(9)), (Coles, dec!(3))]),
        Coles,
        "phu",
    )];
    let planned = plan(
        OrderMode::ColesOnly,
        &items,
        vec![],
        &settings(dec!(0), None, None),
    );

    assert_eq!(planned.options[0].kind, OptionKind::Only(Coles));
    assert_eq!(planned.options[0].disliked.len(), 1);
}
