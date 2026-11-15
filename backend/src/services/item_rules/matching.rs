//! Deciding which rules apply to an item name.
//!
//! A trigger matches when its words appear, in order and next to each other,
//! among the item name's words — ignoring case and punctuation. So
//! "toilet paper" matches "Quilton Toilet-Paper" but not "paper towel", and
//! "milk" does not match "milkshake". Singular and plural forms of a word are
//! the same word ("egg" matches "eggs", "tomato" matches "tomatoes"). There are
//! no built-in synonyms: a rule lists each name it answers to as a trigger.
//!
//! Pure functions only; loading the rules is the caller's business.

use crate::models::db::ItemRule;

/// Splits text into lowercase words, treating anything that is not a letter
/// or digit as a separator.
fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Shortest stem a plural ending may be cut down to, so "as" is not "a".
const MIN_STEM_CHARS: usize = 3;

/// The forms a word may share with its singular or plural: the word itself,
/// and the word with a plural ending ("s", "es", "ies" → "y") taken off.
fn word_forms(word: &str) -> Vec<String> {
    let mut forms = vec![word.to_string()];
    let mut push_stem = |stem: &str, suffix: &str| {
        if stem.chars().count() >= MIN_STEM_CHARS {
            forms.push(format!("{stem}{suffix}"));
        }
    };
    if let Some(stem) = word.strip_suffix("ies") {
        push_stem(stem, "y");
    }
    if let Some(stem) = word.strip_suffix("es") {
        push_stem(stem, "");
    }
    if !word.ends_with("ss") {
        if let Some(stem) = word.strip_suffix('s') {
            push_stem(stem, "");
        }
    }
    forms
}

/// Whether two lowercase words are the same word, allowing one to be the
/// plural of the other.
fn same_word(left: &str, right: &str) -> bool {
    let right_forms = word_forms(right);
    word_forms(left)
        .iter()
        .any(|form| right_forms.contains(form))
}

/// Whether a trigger phrase matches an item name. A trigger with no words at
/// all (only punctuation) matches nothing.
pub fn trigger_matches(trigger: &str, item_name: &str) -> bool {
    let wanted = words(trigger);
    if wanted.is_empty() {
        return false;
    }
    words(item_name).windows(wanted.len()).any(|window| {
        window
            .iter()
            .zip(&wanted)
            .all(|(word, want)| same_word(word, want))
    })
}

/// The rules, in their given order, with at least one trigger matching the
/// item name.
pub fn matching_rules<'r>(item_name: &str, rules: &'r [ItemRule]) -> Vec<&'r ItemRule> {
    rules
        .iter()
        .filter(|rule| {
            rule.triggers
                .iter()
                .any(|trigger| trigger_matches(trigger, item_name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::{matching_rules, trigger_matches};
    use crate::models::db::ItemRule;

    fn rule(triggers: &[&str]) -> ItemRule {
        ItemRule {
            id: Uuid::new_v4(),
            triggers: triggers.iter().map(|t| t.to_string()).collect(),
            filter_terms: vec!["x".to_string()],
            apply_to_manual: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn the_same_name_matches() {
        assert!(trigger_matches("toilet paper", "toilet paper"));
    }

    #[test]
    fn case_whitespace_and_punctuation_are_ignored() {
        assert!(trigger_matches(
            "Toilet  Paper",
            "quilton TOILET-paper, 24 pack"
        ));
    }

    #[test]
    fn a_trigger_can_sit_anywhere_in_the_name() {
        assert!(trigger_matches("milk", "full cream milk"));
        assert!(trigger_matches("full cream", "full cream milk"));
    }

    #[test]
    fn part_of_a_word_is_not_a_match() {
        assert!(!trigger_matches("milk", "milkshake"));
        assert!(!trigger_matches("egg", "eggplant"));
    }

    #[test]
    fn a_plural_matches_its_singular_either_way() {
        assert!(trigger_matches("egg", "free range eggs"));
        assert!(trigger_matches("eggs", "egg"));
        assert!(trigger_matches("tomato", "roma tomatoes"));
        assert!(trigger_matches("berries", "berry"));
        assert!(trigger_matches("peach", "peaches"));
        assert!(trigger_matches("toilet papers", "toilet paper"));
    }

    #[test]
    fn short_words_and_double_s_are_not_treated_as_plurals() {
        assert!(!trigger_matches("as", "a"));
        assert!(!trigger_matches("glass", "glas"));
        assert!(!trigger_matches("bus", "bu"));
    }

    #[test]
    fn the_words_must_be_together_and_in_order() {
        assert!(!trigger_matches("toilet paper", "paper for the toilet"));
        assert!(!trigger_matches("toilet paper", "toilet cleaner and paper"));
    }

    #[test]
    fn a_trigger_longer_than_the_name_does_not_match() {
        assert!(!trigger_matches("full cream milk", "milk"));
    }

    #[test]
    fn a_trigger_of_only_punctuation_matches_nothing() {
        assert!(!trigger_matches("--", "milk -- 2L"));
    }

    #[test]
    fn digits_are_words_too() {
        assert!(trigger_matches("2l milk", "a2 2L milk"));
    }

    #[test]
    fn matching_rules_keeps_the_given_order() {
        let rules = vec![rule(&["milk"]), rule(&["bread"]), rule(&["cream", "milk"])];
        let matched = matching_rules("full cream milk", &rules);
        let ids: Vec<_> = matched.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![rules[0].id, rules[2].id]);
    }

    #[test]
    fn no_rule_matches_an_unrelated_name() {
        let rules = vec![rule(&["milk"])];
        assert!(matching_rules("bread", &rules).is_empty());
    }
}
