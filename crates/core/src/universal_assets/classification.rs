//! The universal asset class enum.
//!
//! Every value below maps to exactly one row family — either an existing
//! Mizan concept (cash, FX), or a typed-extension-table row created
//! alongside the base assets row. The string form is the canonical
//! `assets.classification` column value; tests guarantee round-trip
//! safety so the database is always in a parseable state.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::assets::AssetKind;

/// The 22 universal asset classes from `docs/mizan-smart-plan/PLAN.md`
/// Phase 1 / Prompt 4. The string values are SQL-friendly snake_case
/// and are used as the persisted form on `assets.classification` and
/// in the typed-detail CHECK constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClassification {
    PublicEquity,
    Etf,
    MutualFund,
    FixedIncome,
    Sukuk,
    FixedDeposit,
    Cash,
    RealEstate,
    PrivateEquity,
    PrivateCredit,
    HedgeFund,
    VentureCapital,
    Crypto,
    Commodity,
    Gold,
    Silver,
    Insurance,
    Ulip,
    Pension,
    BusinessOwnership,
    Collectible,
    Liability,
    Custom,
}

impl AssetClassification {
    /// Returns the canonical persisted form. Matches the SQL CHECK
    /// constraints in `2026-05-14-000002_universal_asset_model/up.sql`
    /// where applicable.
    pub const fn as_str(self) -> &'static str {
        match self {
            AssetClassification::PublicEquity => "public_equity",
            AssetClassification::Etf => "etf",
            AssetClassification::MutualFund => "mutual_fund",
            AssetClassification::FixedIncome => "fixed_income",
            AssetClassification::Sukuk => "sukuk",
            AssetClassification::FixedDeposit => "fixed_deposit",
            AssetClassification::Cash => "cash",
            AssetClassification::RealEstate => "real_estate",
            AssetClassification::PrivateEquity => "private_equity",
            AssetClassification::PrivateCredit => "private_credit",
            AssetClassification::HedgeFund => "hedge_fund",
            AssetClassification::VentureCapital => "venture_capital",
            AssetClassification::Crypto => "crypto",
            AssetClassification::Commodity => "commodity",
            AssetClassification::Gold => "gold",
            AssetClassification::Silver => "silver",
            AssetClassification::Insurance => "insurance",
            AssetClassification::Ulip => "ulip",
            AssetClassification::Pension => "pension",
            AssetClassification::BusinessOwnership => "business_ownership",
            AssetClassification::Collectible => "collectible",
            AssetClassification::Liability => "liability",
            AssetClassification::Custom => "custom",
        }
    }

    /// Parses the persisted form. Returns `None` for unknown values so
    /// callers can decide whether to treat the row as legacy/unknown or
    /// to reject it.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "public_equity" => Some(AssetClassification::PublicEquity),
            "etf" => Some(AssetClassification::Etf),
            "mutual_fund" => Some(AssetClassification::MutualFund),
            "fixed_income" => Some(AssetClassification::FixedIncome),
            "sukuk" => Some(AssetClassification::Sukuk),
            "fixed_deposit" => Some(AssetClassification::FixedDeposit),
            "cash" => Some(AssetClassification::Cash),
            "real_estate" => Some(AssetClassification::RealEstate),
            "private_equity" => Some(AssetClassification::PrivateEquity),
            "private_credit" => Some(AssetClassification::PrivateCredit),
            "hedge_fund" => Some(AssetClassification::HedgeFund),
            "venture_capital" => Some(AssetClassification::VentureCapital),
            "crypto" => Some(AssetClassification::Crypto),
            "commodity" => Some(AssetClassification::Commodity),
            "gold" => Some(AssetClassification::Gold),
            "silver" => Some(AssetClassification::Silver),
            "insurance" => Some(AssetClassification::Insurance),
            "ulip" => Some(AssetClassification::Ulip),
            "pension" => Some(AssetClassification::Pension),
            "business_ownership" => Some(AssetClassification::BusinessOwnership),
            "collectible" => Some(AssetClassification::Collectible),
            "liability" => Some(AssetClassification::Liability),
            "custom" => Some(AssetClassification::Custom),
            _ => None,
        }
    }

    /// Returns the typed extension table this class persists detail
    /// rows into, or `None` for classes that need no extension table
    /// (cash, crypto, custom).
    pub const fn detail_table(self) -> Option<&'static str> {
        match self {
            AssetClassification::PublicEquity
            | AssetClassification::Etf
            | AssetClassification::MutualFund => Some("asset_public_market_details"),
            AssetClassification::FixedIncome
            | AssetClassification::Sukuk
            | AssetClassification::FixedDeposit => Some("asset_fixed_income_details"),
            AssetClassification::RealEstate => Some("asset_real_estate_details"),
            AssetClassification::PrivateEquity
            | AssetClassification::PrivateCredit
            | AssetClassification::HedgeFund
            | AssetClassification::VentureCapital => Some("asset_private_investment_details"),
            AssetClassification::Insurance
            | AssetClassification::Ulip
            | AssetClassification::Pension => Some("asset_insurance_details"),
            AssetClassification::Commodity
            | AssetClassification::Gold
            | AssetClassification::Silver => Some("asset_commodity_details"),
            AssetClassification::BusinessOwnership => Some("asset_business_details"),
            AssetClassification::Collectible => Some("asset_collectible_details"),
            AssetClassification::Liability => Some("asset_liability_details"),
            AssetClassification::Cash
            | AssetClassification::Crypto
            | AssetClassification::Custom => None,
        }
    }

    /// Maps a universal class to the legacy `AssetKind` enum that the
    /// existing portfolio/holdings/quotes pipeline reads. Keeping this
    /// mapping in one place is critical: it is how new universal-flow
    /// rows continue to participate in net-worth math without forcing
    /// every consumer of `assets.kind` to learn about the 23-class
    /// classification.
    ///
    /// The legacy enum is coarser than the universal one, so several
    /// universal classes map to the same `AssetKind`. The reverse map
    /// is intentionally one-way: a legacy row without a classification
    /// column value still works; only the universal flow sets it.
    pub const fn to_legacy_kind(self) -> AssetKind {
        match self {
            AssetClassification::PublicEquity
            | AssetClassification::Etf
            | AssetClassification::MutualFund
            | AssetClassification::FixedIncome
            | AssetClassification::Sukuk
            | AssetClassification::FixedDeposit
            | AssetClassification::Cash
            | AssetClassification::Crypto => AssetKind::Investment,
            AssetClassification::RealEstate => AssetKind::Property,
            AssetClassification::PrivateEquity
            | AssetClassification::PrivateCredit
            | AssetClassification::HedgeFund
            | AssetClassification::VentureCapital
            | AssetClassification::BusinessOwnership => AssetKind::PrivateEquity,
            AssetClassification::Commodity
            | AssetClassification::Gold
            | AssetClassification::Silver => AssetKind::PreciousMetal,
            AssetClassification::Collectible => AssetKind::Collectible,
            AssetClassification::Liability => AssetKind::Liability,
            AssetClassification::Insurance
            | AssetClassification::Ulip
            | AssetClassification::Pension
            | AssetClassification::Custom => AssetKind::Other,
        }
    }

    /// Enumerates every class — useful for fixtures and round-trip
    /// tests. Order matches the spec listing.
    pub const fn all() -> [AssetClassification; 23] {
        [
            AssetClassification::PublicEquity,
            AssetClassification::Etf,
            AssetClassification::MutualFund,
            AssetClassification::FixedIncome,
            AssetClassification::Sukuk,
            AssetClassification::FixedDeposit,
            AssetClassification::Cash,
            AssetClassification::RealEstate,
            AssetClassification::PrivateEquity,
            AssetClassification::PrivateCredit,
            AssetClassification::HedgeFund,
            AssetClassification::VentureCapital,
            AssetClassification::Crypto,
            AssetClassification::Commodity,
            AssetClassification::Gold,
            AssetClassification::Silver,
            AssetClassification::Insurance,
            AssetClassification::Ulip,
            AssetClassification::Pension,
            AssetClassification::BusinessOwnership,
            AssetClassification::Collectible,
            AssetClassification::Liability,
            AssetClassification::Custom,
        ]
    }
}

impl fmt::Display for AssetClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_23_classes_round_trip_through_string() {
        for class in AssetClassification::all() {
            let s = class.as_str();
            // No spaces, no uppercase — every value must be safe to drop
            // into a SQL CHECK constraint or a URL slug.
            assert!(
                s.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{:?} -> {:?} is not snake_case",
                class,
                s
            );
            assert_eq!(AssetClassification::parse(s), Some(class));
        }
    }

    #[test]
    fn parse_rejects_unknown_and_legacy_values() {
        assert_eq!(AssetClassification::parse(""), None);
        assert_eq!(AssetClassification::parse("PUBLIC_EQUITY"), None);
        assert_eq!(AssetClassification::parse("stocks"), None);
        assert_eq!(AssetClassification::parse("INVESTMENT"), None); // legacy AssetKind
    }

    #[test]
    fn json_serialisation_uses_snake_case() {
        let json = serde_json::to_string(&AssetClassification::FixedDeposit).unwrap();
        assert_eq!(json, "\"fixed_deposit\"");
        let parsed: AssetClassification = serde_json::from_str("\"fixed_deposit\"").unwrap();
        assert_eq!(parsed, AssetClassification::FixedDeposit);
    }

    #[test]
    fn detail_table_groups_match_typed_extension_schema() {
        // Three fixed-income classes share one table.
        assert_eq!(
            AssetClassification::Sukuk.detail_table(),
            AssetClassification::FixedDeposit.detail_table()
        );
        assert_eq!(
            AssetClassification::FixedIncome.detail_table(),
            Some("asset_fixed_income_details")
        );

        // Four private-investment classes share one table.
        for class in [
            AssetClassification::PrivateEquity,
            AssetClassification::PrivateCredit,
            AssetClassification::HedgeFund,
            AssetClassification::VentureCapital,
        ] {
            assert_eq!(
                class.detail_table(),
                Some("asset_private_investment_details")
            );
        }

        // Cash / crypto / custom have no typed-detail table.
        assert_eq!(AssetClassification::Cash.detail_table(), None);
        assert_eq!(AssetClassification::Crypto.detail_table(), None);
        assert_eq!(AssetClassification::Custom.detail_table(), None);
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", AssetClassification::Sukuk), "sukuk");
    }

    #[test]
    fn legacy_kind_mapping_covers_every_universal_class() {
        // The match in to_legacy_kind is exhaustive, but covering each
        // variant here protects against future drift: any new class
        // added to the enum without a mapping entry fails this test
        // immediately via the compiler-enforced match below.
        for class in AssetClassification::all() {
            let legacy = class.to_legacy_kind();
            // Sanity: the legacy enum is never Investment for assets
            // whose classification is "Liability" — net-worth math
            // depends on the legacy AssetKind for sign.
            if class == AssetClassification::Liability {
                assert_eq!(legacy, AssetKind::Liability);
            }
            // Property class only maps from RealEstate.
            if legacy == AssetKind::Property {
                assert_eq!(class, AssetClassification::RealEstate);
            }
        }
    }

    #[test]
    fn fixed_income_family_maps_to_investment_legacy_kind() {
        // The fixed-income family rides on Investment so existing
        // portfolio math keeps working until the dedicated fixed-income
        // engine lands (Phase 3 P19).
        for class in [
            AssetClassification::FixedIncome,
            AssetClassification::Sukuk,
            AssetClassification::FixedDeposit,
        ] {
            assert_eq!(class.to_legacy_kind(), AssetKind::Investment);
        }
    }

    #[test]
    fn precious_metals_share_legacy_kind() {
        for class in [
            AssetClassification::Commodity,
            AssetClassification::Gold,
            AssetClassification::Silver,
        ] {
            assert_eq!(class.to_legacy_kind(), AssetKind::PreciousMetal);
        }
    }
}
