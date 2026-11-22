//! A typed set of the capabilities one server actually serves.
//!
//! [`Feature`] is the vocabulary; `ActiveFeatures` is the answer. Keeping the answer
//! as a bitset rather than a `Vec<Feature>` makes the type `Copy`, allocation-free and
//! cheap to read on every render, and makes "the set decides membership" the only way
//! to ask. Membership is a bit test, so `contains` cannot allocate and is safe in a
//! wasm render path.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::feature::Feature;

/// The set of [`Feature`]s a server serves, in declaration order.
///
/// One machine word because the protocol names fewer than 64 features; the assertion
/// below keeps that promise honest instead of letting a 65th variant shift silently.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActiveFeatures(u64);

const _: () = assert!(Feature::COUNT <= u64::BITS as usize);

impl ActiveFeatures {
    /// No capability at all.
    pub const NONE: Self = Self(0);

    /// Every capability this build names.
    pub const ALL: Self = Self::from_slice(&Feature::ALL);

    /// Builds a set from a list of features; order does not matter.
    #[must_use]
    pub const fn from_slice(features: &[Feature]) -> Self {
        let mut set = Self::NONE;
        let mut index = 0;
        while index < features.len() {
            set.insert(features[index]);
            index += 1;
        }
        set
    }

    /// Whether `feature` is in the set.
    #[must_use]
    pub const fn contains(self, feature: Feature) -> bool {
        self.0 & Self::bit(feature) != 0
    }

    /// Adds `feature`; inserting twice is harmless.
    pub const fn insert(&mut self, feature: Feature) {
        self.0 |= Self::bit(feature);
    }

    /// Iterates the set in [`Feature::ALL`] declaration order.
    pub fn iter(self) -> impl Iterator<Item = Feature> {
        Feature::ALL
            .into_iter()
            .filter(move |feature| self.contains(*feature))
    }

    /// Whether no feature is active.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// How many features are active.
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    const fn bit(feature: Feature) -> u64 {
        1 << feature.index()
    }
}

/// Serialises as the array of wire names the old `Vec<Feature>` produced, so a stored
/// capability answer keeps its shape.
impl Serialize for ActiveFeatures {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter().map(Feature::as_str))
    }
}

/// Reads either the current name array or an older document; a name this build does not
/// know is dropped here because [`crate::Capabilities::unknown`] retains it separately.
impl<'de> Deserialize<'de> for ActiveFeatures {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let names = Vec::<String>::deserialize(deserializer)?;
        let mut set = Self::NONE;
        for name in names {
            if let Some(feature) = Feature::parse(&name) {
                set.insert(feature);
            }
        }
        Ok(set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_round_trips_every_feature_through_its_bit() {
        for feature in Feature::ALL {
            let mut set = ActiveFeatures::NONE;
            assert!(!set.contains(feature), "{feature:?} started active");
            set.insert(feature);
            assert!(set.contains(feature), "{feature:?} did not stick");
            assert_eq!(set.len(), 1);
        }
    }

    #[test]
    fn iteration_follows_the_declaration_order() {
        let mut set = ActiveFeatures::NONE;
        for feature in [Feature::Favourites, Feature::Admin, Feature::Polls] {
            set.insert(feature);
        }
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![Feature::Admin, Feature::Polls, Feature::Favourites]
        );
    }

    #[test]
    fn an_empty_set_contains_nothing_and_has_no_iterator_items() {
        assert!(ActiveFeatures::NONE.is_empty());
        assert_eq!(ActiveFeatures::NONE.iter().count(), 0);
    }

    #[test]
    fn from_slice_collects_exactly_the_named_features() {
        let set = ActiveFeatures::from_slice(&[Feature::Batch, Feature::Events]);
        assert_eq!(set.len(), 2);
        assert!(set.contains(Feature::Batch));
        assert!(set.contains(Feature::Events));
        assert!(!set.contains(Feature::Polls));
    }

    #[test]
    fn all_contains_every_declared_feature() {
        for feature in Feature::ALL {
            assert!(ActiveFeatures::ALL.contains(feature), "{feature:?}");
        }
        assert_eq!(ActiveFeatures::ALL.len(), Feature::COUNT);
    }
}
