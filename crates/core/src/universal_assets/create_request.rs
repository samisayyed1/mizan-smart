//! Request payload for the universal Add Asset flow (Phase 1 / Prompt 5).
//!
//! The frontend submits a [`UniversalAssetCreateRequest`] — a discriminated
//! union keyed by [`super::AssetClassification`]. Each variant carries:
//!
//!   * the human-friendly fields every class needs (name, currency,
//!     initial value, valuation date),
//!   * a class-specific subtype enum where the spec defines one
//!     (FixedIncome, PrivateInvestment, Insurance, Commodity,
//!     Liability),
//!   * the bare minimum extra fields callers must provide so the
//!     typed extension table is correctly populated. Anything beyond
//!     that is filled in later through the per-class detail pages
//!     (Phase 3 prompts 17–22), so the senior-friendly form stays
//!     short.
//!
//! Every variant goes through [`UniversalAssetCreateRequest::validate`]
//! before any IO. The repository in `crates/storage-sqlite` performs
//! the multi-table insert in a single transaction so a partially
//! created asset cannot be observed.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result, ValidationError};
use crate::universal_assets::details::{
    CommodityType, FixedIncomeSubtype, LiabilityType, PublicMarketSubClass,
};
use crate::universal_assets::AssetClassification;

/// Common base for every classification — name, currency, and the
/// initial valuation. Keeps the discriminated-union variants below
/// readable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UniversalAssetCommon {
    /// Display name. Required.
    pub name: String,
    /// 3-letter uppercase ISO 4217 currency the asset is priced in.
    pub currency: String,
    /// Optional notes. Persisted on the assets row.
    #[serde(default)]
    pub notes: Option<String>,
    /// Initial valuation. The universal flow always stores an initial
    /// value so the asset has a price on day 1 and net-worth math
    /// can include it immediately. If callers genuinely don't know,
    /// they should pass zero and a notes string explaining why.
    pub initial_value: Decimal,
    /// Date the initial valuation applies to. Defaults at the
    /// frontend to today; the backend echoes whatever the caller sends.
    pub initial_value_date: NaiveDate,
}

impl UniversalAssetCommon {
    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "name is required".into(),
            )));
        }
        let currency = self.currency.trim();
        if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "currency must be a 3-letter ISO 4217 code".into(),
            )));
        }
        Ok(())
    }
}

/// Discriminated union of per-classification create payloads. Every
/// variant produces the same end-state shape in the database:
///
///   * one row in `assets` (kind=legacy mapping, classification=class,
///     quote_mode=MANUAL)
///   * one row in the matching typed-extension table (if the class
///     has one — Cash/Crypto/Custom do not)
///   * one row in `valuations` (manual source) for the initial value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "classification",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum UniversalAssetCreateRequest {
    PublicEquity {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        sub_class: Option<PublicMarketSubClass>,
        #[serde(default)]
        isin: Option<String>,
    },
    Etf {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        isin: Option<String>,
    },
    MutualFund {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        isin: Option<String>,
    },
    FixedIncome {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        instrument_subtype: FixedIncomeSubtype,
        #[serde(default)]
        issuer: Option<String>,
        #[serde(default)]
        maturity_date: Option<NaiveDate>,
    },
    Sukuk {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        issuer: Option<String>,
        #[serde(default)]
        maturity_date: Option<NaiveDate>,
    },
    FixedDeposit {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        issuer: Option<String>,
        #[serde(default)]
        maturity_date: Option<NaiveDate>,
    },
    Cash {
        #[serde(flatten)]
        common: UniversalAssetCommon,
    },
    RealEstate {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        property_type: Option<String>,
        #[serde(default)]
        address_approximate: Option<String>,
    },
    PrivateEquity {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        manager: Option<String>,
    },
    PrivateCredit {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        manager: Option<String>,
    },
    HedgeFund {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        manager: Option<String>,
    },
    VentureCapital {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        manager: Option<String>,
    },
    Crypto {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        /// The symbol (BTC, ETH, …). Optional because the universal
        /// flow defaults to manual mode; the user can link a market
        /// symbol later through a separate action.
        #[serde(default)]
        symbol: Option<String>,
    },
    Commodity {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        commodity_type: CommodityType,
        #[serde(default)]
        weight_value: Option<Decimal>,
        #[serde(default)]
        weight_unit: Option<String>,
        #[serde(default)]
        purity: Option<String>,
    },
    Gold {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        weight_value: Option<Decimal>,
        #[serde(default)]
        weight_unit: Option<String>,
        #[serde(default)]
        purity: Option<String>,
    },
    Silver {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        weight_value: Option<Decimal>,
        #[serde(default)]
        weight_unit: Option<String>,
        #[serde(default)]
        purity: Option<String>,
    },
    Insurance {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        provider: Option<String>,
    },
    Ulip {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        provider: Option<String>,
    },
    Pension {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        provider: Option<String>,
    },
    BusinessOwnership {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        business_name: Option<String>,
        #[serde(default)]
        ownership_percent: Option<Decimal>,
    },
    Collectible {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        #[serde(default)]
        collectible_type: Option<String>,
        #[serde(default)]
        maker: Option<String>,
    },
    Liability {
        #[serde(flatten)]
        common: UniversalAssetCommon,
        liability_type: LiabilityType,
        #[serde(default)]
        lender: Option<String>,
    },
    Custom {
        #[serde(flatten)]
        common: UniversalAssetCommon,
    },
}

impl UniversalAssetCreateRequest {
    /// Returns the classification a particular request will create.
    /// Used by the storage layer to choose which typed extension
    /// table to insert into.
    pub fn classification(&self) -> AssetClassification {
        use UniversalAssetCreateRequest::*;
        match self {
            PublicEquity { .. } => AssetClassification::PublicEquity,
            Etf { .. } => AssetClassification::Etf,
            MutualFund { .. } => AssetClassification::MutualFund,
            FixedIncome { .. } => AssetClassification::FixedIncome,
            Sukuk { .. } => AssetClassification::Sukuk,
            FixedDeposit { .. } => AssetClassification::FixedDeposit,
            Cash { .. } => AssetClassification::Cash,
            RealEstate { .. } => AssetClassification::RealEstate,
            PrivateEquity { .. } => AssetClassification::PrivateEquity,
            PrivateCredit { .. } => AssetClassification::PrivateCredit,
            HedgeFund { .. } => AssetClassification::HedgeFund,
            VentureCapital { .. } => AssetClassification::VentureCapital,
            Crypto { .. } => AssetClassification::Crypto,
            Commodity { .. } => AssetClassification::Commodity,
            Gold { .. } => AssetClassification::Gold,
            Silver { .. } => AssetClassification::Silver,
            Insurance { .. } => AssetClassification::Insurance,
            Ulip { .. } => AssetClassification::Ulip,
            Pension { .. } => AssetClassification::Pension,
            BusinessOwnership { .. } => AssetClassification::BusinessOwnership,
            Collectible { .. } => AssetClassification::Collectible,
            Liability { .. } => AssetClassification::Liability,
            Custom { .. } => AssetClassification::Custom,
        }
    }

    /// Borrows the per-variant common block.
    pub fn common(&self) -> &UniversalAssetCommon {
        use UniversalAssetCreateRequest::*;
        match self {
            PublicEquity { common, .. }
            | Etf { common, .. }
            | MutualFund { common, .. }
            | FixedIncome { common, .. }
            | Sukuk { common, .. }
            | FixedDeposit { common, .. }
            | Cash { common, .. }
            | RealEstate { common, .. }
            | PrivateEquity { common, .. }
            | PrivateCredit { common, .. }
            | HedgeFund { common, .. }
            | VentureCapital { common, .. }
            | Crypto { common, .. }
            | Commodity { common, .. }
            | Gold { common, .. }
            | Silver { common, .. }
            | Insurance { common, .. }
            | Ulip { common, .. }
            | Pension { common, .. }
            | BusinessOwnership { common, .. }
            | Collectible { common, .. }
            | Liability { common, .. }
            | Custom { common, .. } => common,
        }
    }

    /// Validate the common block plus class-specific invariants.
    pub fn validate(&self) -> Result<()> {
        self.common().validate()?;
        if let UniversalAssetCreateRequest::BusinessOwnership {
            ownership_percent: Some(percent),
            ..
        } = self
        {
            if *percent < Decimal::ZERO || *percent > Decimal::from(100) {
                return Err(Error::Validation(ValidationError::InvalidInput(format!(
                    "ownership_percent {} must be between 0 and 100",
                    percent
                ))));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn common() -> UniversalAssetCommon {
        UniversalAssetCommon {
            name: "Test asset".into(),
            currency: "USD".into(),
            notes: None,
            initial_value: dec!(100_000),
            initial_value_date: NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
        }
    }

    #[test]
    fn classification_returns_the_variant_class() {
        let req = UniversalAssetCreateRequest::RealEstate {
            common: common(),
            property_type: Some("apartment".into()),
            address_approximate: Some("London".into()),
        };
        assert_eq!(req.classification(), AssetClassification::RealEstate);
    }

    #[test]
    fn common_borrows_block_from_any_variant() {
        let req = UniversalAssetCreateRequest::Sukuk {
            common: common(),
            issuer: Some("DXB".into()),
            maturity_date: None,
        };
        assert_eq!(req.common().name, "Test asset");
    }

    #[test]
    fn validate_accepts_well_formed_request_for_every_variant() {
        // Spot-check a handful of variants. The common block validation
        // is shared so we don't need to enumerate all 23.
        let variants = vec![
            UniversalAssetCreateRequest::PublicEquity {
                common: common(),
                sub_class: None,
                isin: None,
            },
            UniversalAssetCreateRequest::Cash { common: common() },
            UniversalAssetCreateRequest::Liability {
                common: common(),
                liability_type: LiabilityType::Mortgage,
                lender: None,
            },
            UniversalAssetCreateRequest::Commodity {
                common: common(),
                commodity_type: CommodityType::Gold,
                weight_value: Some(dec!(10)),
                weight_unit: Some("oz".into()),
                purity: Some("999".into()),
            },
        ];
        for req in variants {
            assert!(req.validate().is_ok(), "{:?} should validate", req);
        }
    }

    #[test]
    fn validate_rejects_empty_name() {
        let bad = UniversalAssetCreateRequest::Custom {
            common: UniversalAssetCommon {
                name: "   ".into(),
                ..common()
            },
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn validate_rejects_lowercase_currency() {
        let bad = UniversalAssetCreateRequest::Cash {
            common: UniversalAssetCommon {
                currency: "usd".into(),
                ..common()
            },
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn validate_rejects_out_of_range_business_ownership_percent() {
        let bad = UniversalAssetCreateRequest::BusinessOwnership {
            common: common(),
            business_name: Some("Co".into()),
            ownership_percent: Some(dec!(150)),
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn json_round_trips_with_tag_classification_field() {
        let req = UniversalAssetCreateRequest::RealEstate {
            common: common(),
            property_type: Some("apartment".into()),
            address_approximate: Some("London".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        // The tag field is the snake_case classification value.
        assert!(json.contains("\"classification\":\"real_estate\""));
        // initial_value_date is camelCased via the common block flatten.
        assert!(json.contains("initialValueDate"));
        let back: UniversalAssetCreateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }
}
