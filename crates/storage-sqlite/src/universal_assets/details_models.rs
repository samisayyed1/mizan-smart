//! Diesel `Insertable` rows for the nine typed extension tables.
//!
//! These are intentionally lightweight — `Insertable` only, with a
//! single `new()` helper that pins `created_at` / `updated_at` from a
//! caller-supplied timestamp. Read paths and per-class repositories
//! land alongside the Phase 3 prompts that need them (P17 private
//! investments, P19 fixed income, P21 corporate actions, etc.).
//!
//! Every struct populates only the columns the universal create flow
//! requires; the rest stay NULL until the per-class detail page
//! deepens them.

use chrono::{DateTime, Utc};
use diesel::prelude::*;

fn rfc(now: DateTime<Utc>) -> String {
    now.to_rfc3339()
}

// ============================================================================
// asset_public_market_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_public_market_details)]
pub struct InsertablePublicMarketDetails {
    pub asset_id: String,
    pub sub_class: Option<String>,
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub figi: Option<String>,
    pub expense_ratio_bps: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertablePublicMarketDetails {
    pub fn new(
        asset_id: String,
        sub_class: Option<String>,
        isin: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            sub_class,
            isin,
            cusip: None,
            figi: None,
            expense_ratio_bps: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_fixed_income_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_fixed_income_details)]
pub struct InsertableFixedIncomeDetails {
    pub asset_id: String,
    pub instrument_subtype: String,
    pub issuer: Option<String>,
    pub isin: Option<String>,
    pub face_value: Option<String>,
    pub currency: Option<String>,
    pub purchase_date: Option<String>,
    pub maturity_date: Option<String>,
    pub coupon_or_profit_rate: Option<String>,
    pub payment_frequency: Option<String>,
    pub day_count_convention: Option<String>,
    pub is_sukuk: i32,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableFixedIncomeDetails {
    pub fn new(
        asset_id: String,
        instrument_subtype: String,
        issuer: Option<String>,
        currency: Option<String>,
        maturity_date: Option<String>,
        is_sukuk: bool,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            instrument_subtype,
            issuer,
            isin: None,
            face_value: None,
            currency,
            purchase_date: None,
            maturity_date,
            coupon_or_profit_rate: None,
            payment_frequency: None,
            day_count_convention: None,
            is_sukuk: if is_sukuk { 1 } else { 0 },
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_real_estate_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_real_estate_details)]
pub struct InsertableRealEstateDetails {
    pub asset_id: String,
    pub property_type: Option<String>,
    pub address_approximate: Option<String>,
    pub address_exact: Option<String>,
    pub size_value: Option<String>,
    pub size_unit: Option<String>,
    pub bedrooms: Option<i32>,
    pub purchase_date: Option<String>,
    pub purchase_price: Option<String>,
    pub purchase_currency: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableRealEstateDetails {
    pub fn new(
        asset_id: String,
        property_type: Option<String>,
        address_approximate: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            property_type,
            address_approximate,
            address_exact: None,
            size_value: None,
            size_unit: None,
            bedrooms: None,
            purchase_date: None,
            purchase_price: None,
            purchase_currency: None,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_private_investment_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_private_investment_details)]
pub struct InsertablePrivateInvestmentDetails {
    pub asset_id: String,
    pub instrument_subtype: String,
    pub manager: Option<String>,
    pub strategy: Option<String>,
    pub vintage_year: Option<i32>,
    pub commitment_amount: Option<String>,
    pub commitment_currency: Option<String>,
    pub inception_date: Option<String>,
    pub notes: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertablePrivateInvestmentDetails {
    pub fn new(
        asset_id: String,
        instrument_subtype: String,
        manager: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            instrument_subtype,
            manager,
            strategy: None,
            vintage_year: None,
            commitment_amount: None,
            commitment_currency: None,
            inception_date: None,
            notes: None,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_insurance_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_insurance_details)]
pub struct InsertableInsuranceDetails {
    pub asset_id: String,
    pub policy_type: String,
    pub provider: Option<String>,
    pub policy_number_hash: Option<String>,
    pub start_date: Option<String>,
    pub maturity_date: Option<String>,
    pub premium_amount: Option<String>,
    pub premium_currency: Option<String>,
    pub payment_frequency: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableInsuranceDetails {
    pub fn new(
        asset_id: String,
        policy_type: String,
        provider: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            policy_type,
            provider,
            policy_number_hash: None,
            start_date: None,
            maturity_date: None,
            premium_amount: None,
            premium_currency: None,
            payment_frequency: None,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_commodity_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_commodity_details)]
pub struct InsertableCommodityDetails {
    pub asset_id: String,
    pub commodity_type: String,
    pub weight_value: Option<String>,
    pub weight_unit: Option<String>,
    pub purity: Option<String>,
    pub storage_location: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableCommodityDetails {
    pub fn new(
        asset_id: String,
        commodity_type: String,
        weight_value: Option<String>,
        weight_unit: Option<String>,
        purity: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            commodity_type,
            weight_value,
            weight_unit,
            purity,
            storage_location: None,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_business_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_business_details)]
pub struct InsertableBusinessDetails {
    pub asset_id: String,
    pub business_name: Option<String>,
    pub ownership_percent: Option<String>,
    pub legal_form: Option<String>,
    pub country: Option<String>,
    pub incorporation_date: Option<String>,
    pub notes: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableBusinessDetails {
    pub fn new(
        asset_id: String,
        business_name: Option<String>,
        ownership_percent: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            business_name,
            ownership_percent,
            legal_form: None,
            country: None,
            incorporation_date: None,
            notes: None,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_collectible_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_collectible_details)]
pub struct InsertableCollectibleDetails {
    pub asset_id: String,
    pub collectible_type: Option<String>,
    pub maker: Option<String>,
    pub model_reference: Option<String>,
    pub year: Option<i32>,
    pub condition: Option<String>,
    pub has_box: i32,
    pub has_papers: i32,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableCollectibleDetails {
    pub fn new(
        asset_id: String,
        collectible_type: Option<String>,
        maker: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            collectible_type,
            maker,
            model_reference: None,
            year: None,
            condition: None,
            has_box: 0,
            has_papers: 0,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}

// ============================================================================
// asset_liability_details
// ============================================================================

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_liability_details)]
pub struct InsertableLiabilityDetails {
    pub asset_id: String,
    pub liability_type: String,
    pub lender: Option<String>,
    pub principal_original: Option<String>,
    pub principal_currency: Option<String>,
    pub interest_rate: Option<String>,
    pub interest_compounding: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub linked_asset_id: Option<String>,
    pub source_citation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InsertableLiabilityDetails {
    pub fn new(
        asset_id: String,
        liability_type: String,
        lender: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        let stamp = rfc(now);
        Self {
            asset_id,
            liability_type,
            lender,
            principal_original: None,
            principal_currency: None,
            interest_rate: None,
            interest_compounding: None,
            start_date: None,
            end_date: None,
            linked_asset_id: None,
            source_citation_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }
    }
}
