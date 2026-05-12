//! Smart CSV column detection for broker imports.
//!
//! # What this module does
//!
//! Mizan accepts CSV exports from a long tail of brokers, each with
//! their own column names and ordering. The naive approach — match
//! column headers against a list of synonyms — falls over the moment
//! a header is ambiguous ("Price" on a Yahoo Portfolio CSV means the
//! live quote, NOT the purchase price the user paid) or missing ("Total"
//! could be the trade amount or the daily volume).
//!
//! This module replaces that with a **multi-signal** detector that
//! combines three independent sources of evidence:
//!
//! 1. **Broker fingerprinting** — known broker CSV layouts are
//!    recognised by their full header signature and mapped via a
//!    pre-built template. This is the highest-confidence path: zero
//!    inference, just a lookup. Covers the most common brokers
//!    Mizan users actually upload (Yahoo Portfolio, Zerodha
//!    Tradebook, Interactive Brokers Flex, Schwab, Wealthsimple,
//!    Robinhood). Fingerprints match a *subset* of expected headers
//!    rather than exact equality, because brokers regularly add
//!    optional columns between releases.
//!
//! 2. **Column data profiling** — even when no fingerprint matches,
//!    each column is profiled by sniffing the first ~50 data rows
//!    and classifying as Date / Numeric / Symbol / Currency /
//!    ActivityType / Text. A column's data type either confirms or
//!    contradicts what the header text suggests. A header that reads
//!    "Price" but contains values like "AAPL", "MSFT" is clearly NOT
//!    a price column — even though the legacy header matcher would
//!    map it as one.
//!
//! 3. **Header text matching** — the legacy pattern-based approach,
//!    kept as a third signal. Most-specific-first ordering so
//!    "purchase price" beats "price", "trade date" beats "date".
//!
//! Each candidate (column, field) pair gets a scalar score combining
//! all three signals. The assignment is then solved greedily by
//! descending score, ensuring no column is double-claimed.
//!
//! # When this module wins over the old `auto_detect_field_mappings`
//!
//! - **Yahoo Portfolio CSV**: data profiling distinguishes "Current
//!   Price" (numeric, varies by query time) from "Purchase Price"
//!   (numeric, stable). Fingerprint handles it directly.
//! - **Zerodha Tradebook**: column order is fixed but the header
//!   spelling differs across years; fingerprint matches the stable
//!   subset.
//! - **IBKR Flex Query**: report header line precedes the data; the
//!   profile picks the right "Quantity" out of multiple numeric
//!   columns by checking which has consistent integer values.
//! - **Anything with weird non-English column names**: the data
//!   profile is language-agnostic, so even when no header pattern
//!   matches, the dates / numerics / symbols get picked up.
//!
//! # Design choices kept light
//!
//! - No ML, no embeddings, no external service. Heuristics only.
//!   This is a local-first app; everything has to run offline on
//!   the user's machine without bloating the binary.
//! - All decisions are **explainable**: every assignment carries
//!   the signals that drove it, so the UI can flag low-confidence
//!   mappings and the user can override.
//! - Backwards compatible: the existing `auto_detect_field_mappings`
//!   remains as a fallback for when neither fingerprint nor profile
//!   produces a high-confidence pick.

use std::collections::HashMap;

use super::import_csv::{
    FIELD_ACCOUNT, FIELD_ACTIVITY_TYPE, FIELD_AMOUNT, FIELD_COMMENT, FIELD_CURRENCY, FIELD_DATE,
    FIELD_FEE, FIELD_FX_RATE, FIELD_QUANTITY, FIELD_SUBTYPE, FIELD_SYMBOL, FIELD_UNIT_PRICE,
};

// ─────────────────────────────────────────────────────────────────────
// Column profile — what we infer about each CSV column from its data
// ─────────────────────────────────────────────────────────────────────

/// The inferred semantic type of a column's data.
///
/// Inferred from the first ~50 non-empty rows. Each variant captures
/// what the values "look like" — independent of the column header.
/// A column where every value parses as an ISO-8601 date is
/// `ColumnDataType::Date` regardless of whether the header reads
/// "Date", "Trade Date", "Datum", or anything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnDataType {
    /// Parses as a calendar date in any common format.
    Date,
    /// Parses cleanly as a finite number. Captures BOTH thousands-
    /// separator and decimal-separator behaviour seen.
    Numeric,
    /// 1–6 uppercase letters, optionally with an exchange suffix
    /// like ".L", ".TO", ".NS", or "/". Matches AAPL, RELIANCE.NS,
    /// BTC/USD, etc. Rejects sentence-cased text.
    Symbol,
    /// 3-letter ISO currency code (USD, EUR, INR …) or a known
    /// symbol like "$", "₹", "€".
    Currency,
    /// Words from the activity-type vocabulary (BUY, SELL, DIV,
    /// DIVIDEND, INTEREST, FEE, DEPOSIT, …) or their broker-specific
    /// synonyms.
    ActivityType,
    /// Doesn't fit anything specific.
    Text,
    /// Column is empty / never had data.
    Empty,
}

/// Statistics about a single column's contents.
///
/// `kind` is the dominant inferred type. `kind_confidence` is the
/// share of non-empty rows that match `kind` — 1.0 = every value
/// fits, 0.0 = nothing fit (only possible if every cell was empty,
/// in which case `kind == Empty`).
#[derive(Debug, Clone)]
pub struct ColumnProfile {
    /// The original header text as it appears in the CSV.
    pub header: String,
    /// Lowercased / trimmed header for matching.
    pub header_lower: String,
    /// Inferred data type.
    pub kind: ColumnDataType,
    /// Fraction of sampled non-empty rows that match `kind`.
    /// Range \[0.0, 1.0\].
    pub kind_confidence: f32,
    /// How many non-empty rows we actually looked at. Useful for
    /// downstream code that wants to weight low-sample columns less.
    pub samples_seen: usize,
    /// Whether numeric values in this column consistently look like
    /// whole numbers (no decimals, no fractions). Used to prefer
    /// integer-valued columns for `quantity` over price columns.
    pub looks_integer: bool,
}

impl ColumnProfile {
    /// Infer the profile for a single column from its sampled values.
    ///
    /// `values` should be the raw cell strings from the first N
    /// non-empty data rows (the helper [`build_profiles`] supplies them).
    /// Empty strings are filtered before classification — they tell us
    /// nothing about the column's type.
    fn infer(header: &str, values: &[String]) -> Self {
        let header_lower = header.trim().to_lowercase();

        let non_empty: Vec<&str> = values
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if non_empty.is_empty() {
            return Self {
                header: header.to_string(),
                header_lower,
                kind: ColumnDataType::Empty,
                kind_confidence: 0.0,
                samples_seen: 0,
                looks_integer: false,
            };
        }

        // Classify each cell, then take the dominant kind. We use a
        // simple plurality vote — the kind with the most matches wins.
        // Tie-breaks favour more specific kinds (Currency > Symbol >
        // ActivityType > Date > Numeric > Text) to avoid degenerate
        // cases where, say, "USD" both reads as a currency code and a
        // text token.
        let mut counts: HashMap<ColumnDataType, usize> = HashMap::new();
        let mut numeric_with_decimal = 0usize;
        let mut numeric_total = 0usize;

        for cell in &non_empty {
            let kind = classify_cell(cell);
            *counts.entry(kind).or_insert(0) += 1;
            if kind == ColumnDataType::Numeric {
                numeric_total += 1;
                if cell.contains('.') || cell.contains(',') {
                    // Treat anything with a thousands or decimal
                    // separator as "has decimals" — heuristic but
                    // good enough for distinguishing share counts
                    // (123) from prices (123.45) and amounts
                    // (1,234.56).
                    let post_sep = cell.replace(['$', '€', '₹', '£', ' ', '\u{00A0}'], "");
                    if has_fractional_part(&post_sep) {
                        numeric_with_decimal += 1;
                    }
                }
            }
        }

        let (kind, kind_confidence) = pick_dominant_kind(&counts, non_empty.len());

        let looks_integer =
            kind == ColumnDataType::Numeric && numeric_total > 0 && numeric_with_decimal == 0;

        Self {
            header: header.to_string(),
            header_lower,
            kind,
            kind_confidence,
            samples_seen: non_empty.len(),
            looks_integer,
        }
    }
}

/// Inspect the count table and return (kind, confidence).
///
/// Confidence is the fraction of sampled non-empty values that match
/// the chosen kind. Ties are broken by specificity ordering — see
/// [`ColumnProfile::infer`] for rationale.
fn pick_dominant_kind(
    counts: &HashMap<ColumnDataType, usize>,
    total: usize,
) -> (ColumnDataType, f32) {
    if total == 0 {
        return (ColumnDataType::Empty, 0.0);
    }

    const SPECIFICITY_ORDER: &[ColumnDataType] = &[
        ColumnDataType::Currency,
        ColumnDataType::Symbol,
        ColumnDataType::ActivityType,
        ColumnDataType::Date,
        ColumnDataType::Numeric,
        ColumnDataType::Text,
        ColumnDataType::Empty,
    ];

    let max_count = counts.values().copied().max().unwrap_or(0);
    if max_count == 0 {
        return (ColumnDataType::Text, 0.0);
    }

    // Among all kinds tied for the top count, prefer the most specific.
    let kind = SPECIFICITY_ORDER
        .iter()
        .find(|k| counts.get(*k).copied().unwrap_or(0) == max_count)
        .copied()
        .unwrap_or(ColumnDataType::Text);

    (kind, max_count as f32 / total as f32)
}

/// Classify a single cell value.
///
/// Order matters: tested kinds are checked from most specific to
/// least specific so that values which validly match multiple types
/// (e.g. "USD" is both a Currency token and a Text token) latch onto
/// the more useful one.
fn classify_cell(raw: &str) -> ColumnDataType {
    let s = raw.trim();

    // Currency code or symbol — short, well-defined alphabet.
    if is_currency_token(s) {
        return ColumnDataType::Currency;
    }

    // Activity-type keyword (BUY / SELL / DIV / …). Yes, "SELL"
    // would also pass is_symbol() — but the activity-type check
    // happens first so it wins.
    if is_activity_type_token(s) {
        return ColumnDataType::ActivityType;
    }

    if is_date(s) {
        return ColumnDataType::Date;
    }

    if is_numeric(s) {
        return ColumnDataType::Numeric;
    }

    if is_symbol(s) {
        return ColumnDataType::Symbol;
    }

    ColumnDataType::Text
}

// ─────────────────────────────────────────────────────────────────────
// Single-cell type predicates
// ─────────────────────────────────────────────────────────────────────

/// True if `s` looks like a 3-letter ISO currency code or a known
/// currency symbol. Case-insensitive.
fn is_currency_token(s: &str) -> bool {
    const SYMBOLS: &[&str] = &["$", "€", "£", "¥", "₹", "₩", "₿"];
    if SYMBOLS.iter().any(|sym| s == *sym) {
        return true;
    }
    if s.len() != 3 || !s.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    // Whitelist — anything else 3-letter all-alpha is more likely a
    // ticker than a currency. This list is the set of currencies
    // Mizan's FX layer commonly handles. Expanding it is cheap.
    const KNOWN: &[&str] = &[
        "USD", "EUR", "GBP", "JPY", "CHF", "AUD", "CAD", "NZD", "SEK", "NOK", "DKK", "HKD", "SGD",
        "INR", "CNY", "CNH", "KRW", "TWD", "MXN", "BRL", "ZAR", "RUB", "TRY", "AED", "SAR", "QAR",
        "KWD", "BHD", "OMR", "JOD", "IDR", "MYR", "THB", "PHP", "VND", "PLN", "CZK", "HUF", "ILS",
        "EGP", "PKR", "BDT", "LKR", "BTC", "ETH",
    ];
    KNOWN.iter().any(|c| s.eq_ignore_ascii_case(c))
}

/// True if `s` is an activity-type keyword used by the brokers we
/// know about. Case-insensitive. Synonym-aware.
fn is_activity_type_token(s: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        // Trades
        "buy",
        "sell",
        "purchase",
        "sale",
        "long buy",
        "short sell",
        "cover",
        // Cash
        "deposit",
        "withdrawal",
        "withdraw",
        "transfer",
        "transfer in",
        "transfer out",
        "credit",
        "debit",
        "cash in",
        "cash out",
        "funding",
        // Income
        "dividend",
        "div",
        "interest",
        "int",
        "drip",
        "qualified dividend",
        "ordinary dividend",
        "non-cash dividend",
        "staking reward",
        "yield",
        // Adjustments
        "fee",
        "fees",
        "commission",
        "tax",
        "tds",
        "stt",
        "gst",
        "split",
        "stock split",
        "spin-off",
        "merger",
        "bonus",
        "rebate",
        "refund",
        "expiry",
        "option expiry",
        "assignment",
        "exercise",
    ];
    let lower = s.to_lowercase();
    KEYWORDS.iter().any(|k| *k == lower)
}

/// True if `s` parses as a date in any of the formats brokers use
/// in real exports. Permissive — we're just trying to figure out
/// the column's semantic type, not parse the date for storage.
fn is_date(s: &str) -> bool {
    // Strip any time portion before testing — brokers love to glue
    // "2024-01-15 09:30:00" together.
    let head = s.split_whitespace().next().unwrap_or(s);
    let head = head.split('T').next().unwrap_or(head);
    let parts: Vec<&str> = head
        .split(|c: char| c == '-' || c == '/' || c == '.')
        .collect();
    if parts.len() != 3 {
        return false;
    }
    let all_numeric = parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
    if !all_numeric {
        return false;
    }
    let lens: Vec<usize> = parts.iter().map(|p| p.len()).collect();
    // Accept YYYY-?-?, ?-?-YYYY, plus a handful of two-digit-year
    // formats. Reject anything where no part is plausibly a year.
    matches!(
        lens.as_slice(),
        [4, 1..=2, 1..=2] | [1..=2, 1..=2, 4] | [1..=2, 1..=2, 2]
    )
}

/// True if `s` parses cleanly as a number after stripping currency
/// symbols, sign markers, and thousands separators.
fn is_numeric(s: &str) -> bool {
    let stripped: String = s
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '$' | '€' | '£' | '¥' | '₹' | '₩' | '₿' | ' ' | '\u{00A0}'
            )
        })
        .collect();
    let stripped = stripped.trim();
    let stripped = stripped.trim_start_matches(['+', '-']);
    let stripped = stripped.trim_matches(|c: char| c == '(' || c == ')');
    if stripped.is_empty() {
        return false;
    }

    // Try US (1,234.56) and European (1.234,56) conventions.
    let us = stripped.replace(',', "");
    if us.parse::<f64>().is_ok() {
        return true;
    }
    let euro = stripped.replace('.', "").replace(',', ".");
    euro.parse::<f64>().is_ok()
}

/// True if `s` looks like a ticker / instrument symbol.
///
/// Real-world ticker shapes we need to accept:
///   AAPL                US equity
///   BRK.B / BRK-B       US class shares
///   VOO                 ETF
///   RELIANCE.NS         NSE India
///   0700.HK / 9988.HK   Hong Kong (numeric prefix, alpha suffix)
///   BTC/USD             Crypto pairs
///
/// The previous `letters >= digits` constraint excluded HK tickers
/// where four digits dominate. Replaced with a structural check:
/// must contain at least one uppercase letter, no lowercase, and
/// only the allowed punctuation set. That's loose enough to cover
/// every shape above and strict enough to reject sentence text and
/// pure numeric ids.
fn is_symbol(s: &str) -> bool {
    if s.is_empty() || s.len() > 20 {
        return false;
    }
    let mut letters = 0usize;
    let mut digits = 0usize;
    let mut allowed_punct = 0usize;
    for c in s.chars() {
        match c {
            'A'..='Z' => letters += 1,
            'a'..='z' => return false, // Symbols are uppercase in broker CSVs.
            '0'..='9' => digits += 1,
            '.' | '/' | '-' | ':' | '_' => allowed_punct += 1,
            _ => return false,
        }
    }
    if letters == 0 {
        return false;
    }
    letters + digits + allowed_punct == s.len()
}

/// True if the numeric string `s` (already with currency/space
/// stripped) has a fractional part. Handles both US and European
/// conventions by checking the LAST separator only.
fn has_fractional_part(s: &str) -> bool {
    let last_dot = s.rfind('.');
    let last_comma = s.rfind(',');
    let last_sep = match (last_dot, last_comma) {
        (Some(d), Some(c)) => Some(d.max(c)),
        (Some(d), None) => Some(d),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };
    let Some(idx) = last_sep else {
        return false;
    };
    let tail = &s[idx + 1..];
    // 1–4 digits after the last separator = decimal part. Longer
    // tails (5+) are almost always grouped digits ("1.234.567"),
    // not a fractional part.
    !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) && tail.len() <= 4
}

// ─────────────────────────────────────────────────────────────────────
// Broker fingerprinting — match known CSV layouts directly
// ─────────────────────────────────────────────────────────────────────

/// A pre-built mapping for a recognised broker CSV format.
///
/// `required_headers` is the minimum set of headers that must be
/// present (case-insensitive substring match) to claim this is a
/// match. `mappings` is the canonical column → field assignment.
struct BrokerTemplate {
    /// Human-readable label, shown in logs.
    name: &'static str,
    /// Lowercased required substrings — ALL must appear in headers.
    required_headers: &'static [&'static str],
    /// (field, lowercased header substring to find) mappings.
    field_to_header: &'static [(&'static str, &'static str)],
}

/// All known broker CSV templates, ordered by specificity.
///
/// The matcher walks this list and returns the first template whose
/// `required_headers` ALL appear in the CSV's lowercased headers.
/// More-specific templates come first so that, say, Zerodha's
/// Tradebook (which has both "trade date" and "stt") matches before
/// the more permissive Yahoo template would.
const BROKER_TEMPLATES: &[BrokerTemplate] = &[
    // Zerodha Tradebook — Indian retail; STT is the giveaway since
    // no other broker reports the Indian Securities Transaction Tax.
    BrokerTemplate {
        name: "Zerodha Tradebook",
        required_headers: &[
            "trade date",
            "symbol",
            "trade type",
            "quantity",
            "price",
            "stt",
        ],
        field_to_header: &[
            (FIELD_DATE, "trade date"),
            (FIELD_SYMBOL, "symbol"),
            (FIELD_ACTIVITY_TYPE, "trade type"),
            (FIELD_QUANTITY, "quantity"),
            (FIELD_UNIT_PRICE, "price"),
            (FIELD_FEE, "brokerage"),
        ],
    },
    // Yahoo Finance Portfolio (downloaded CSV). Includes both live
    // quote columns and the user's actual trade data — the trade
    // columns are what we want.
    BrokerTemplate {
        name: "Yahoo Finance Portfolio",
        required_headers: &["symbol", "current price", "trade date", "purchase price"],
        field_to_header: &[
            (FIELD_DATE, "trade date"),
            (FIELD_SYMBOL, "symbol"),
            (FIELD_QUANTITY, "shares"),
            (FIELD_UNIT_PRICE, "purchase price"),
            (FIELD_FEE, "commission"),
            (FIELD_COMMENT, "comment"),
        ],
    },
    // Interactive Brokers Flex Query — desktop / Trader Workstation
    // export. Distinct because it has "TradeID" alongside "Symbol".
    BrokerTemplate {
        name: "Interactive Brokers Flex",
        required_headers: &["tradeid", "symbol", "datetime", "quantity", "tradeprice"],
        field_to_header: &[
            (FIELD_DATE, "datetime"),
            (FIELD_SYMBOL, "symbol"),
            (FIELD_QUANTITY, "quantity"),
            (FIELD_UNIT_PRICE, "tradeprice"),
            (FIELD_AMOUNT, "tradeamount"),
            (FIELD_FEE, "commission"),
            (FIELD_CURRENCY, "currency"),
        ],
    },
    // Charles Schwab transactions export.
    BrokerTemplate {
        name: "Charles Schwab",
        required_headers: &["date", "action", "symbol", "quantity", "price"],
        field_to_header: &[
            (FIELD_DATE, "date"),
            (FIELD_ACTIVITY_TYPE, "action"),
            (FIELD_SYMBOL, "symbol"),
            (FIELD_QUANTITY, "quantity"),
            (FIELD_UNIT_PRICE, "price"),
            (FIELD_FEE, "fees & comm"),
            (FIELD_AMOUNT, "amount"),
        ],
    },
    // Wealthsimple transactions export (Canadian).
    BrokerTemplate {
        name: "Wealthsimple",
        required_headers: &["date", "transaction", "description", "amount"],
        field_to_header: &[
            (FIELD_DATE, "date"),
            (FIELD_ACTIVITY_TYPE, "transaction"),
            (FIELD_AMOUNT, "amount"),
            (FIELD_COMMENT, "description"),
            (FIELD_CURRENCY, "currency"),
        ],
    },
    // Robinhood account statement export.
    BrokerTemplate {
        name: "Robinhood",
        required_headers: &[
            "activity date",
            "trans code",
            "instrument",
            "quantity",
            "price",
        ],
        field_to_header: &[
            (FIELD_DATE, "activity date"),
            (FIELD_ACTIVITY_TYPE, "trans code"),
            (FIELD_SYMBOL, "instrument"),
            (FIELD_QUANTITY, "quantity"),
            (FIELD_UNIT_PRICE, "price"),
            (FIELD_AMOUNT, "amount"),
            (FIELD_COMMENT, "description"),
        ],
    },
];

/// Try to recognise a known broker CSV from its headers.
///
/// Returns the matching template's name (for telemetry / logging)
/// and the resolved column mapping, or `None` when no template
/// matches confidently.
pub fn recognise_broker(headers: &[String]) -> Option<(&'static str, HashMap<String, String>)> {
    let lowered: Vec<String> = headers.iter().map(|h| h.trim().to_lowercase()).collect();

    for tpl in BROKER_TEMPLATES {
        let all_present = tpl
            .required_headers
            .iter()
            .all(|req| lowered.iter().any(|h| h.contains(req)));
        if !all_present {
            continue;
        }

        let mut mapping: HashMap<String, String> = HashMap::new();
        for (field, needle) in tpl.field_to_header {
            if let Some((idx, _)) = lowered.iter().enumerate().find(|(_, h)| h.contains(needle)) {
                mapping.insert((*field).to_string(), headers[idx].clone());
            }
        }

        // Require at least 3 fields resolved to consider it a real
        // template hit — otherwise it's just header noise.
        if mapping.len() >= 3 {
            return Some((tpl.name, mapping));
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────
// Multi-signal detector
// ─────────────────────────────────────────────────────────────────────

/// Build a column profile for every header from the supplied sample
/// rows. `sample_rows` is the parsed CSV data rows (not headers).
///
/// Robust to ragged rows — short rows contribute empty cells for the
/// missing columns rather than panicking.
pub fn build_profiles(headers: &[String], sample_rows: &[Vec<String>]) -> Vec<ColumnProfile> {
    headers
        .iter()
        .enumerate()
        .map(|(col_idx, header)| {
            let column_values: Vec<String> = sample_rows
                .iter()
                .filter_map(|row| row.get(col_idx).cloned())
                .collect();
            ColumnProfile::infer(header, &column_values)
        })
        .collect()
}

/// What kind of data each canonical Mizan field is expected to look
/// like. Used to score column ↔ field affinity from the data side.
fn expected_kind_for_field(field: &str) -> &'static [ColumnDataType] {
    match field {
        f if f == FIELD_DATE => &[ColumnDataType::Date],
        f if f == FIELD_SYMBOL => &[ColumnDataType::Symbol],
        f if f == FIELD_QUANTITY => &[ColumnDataType::Numeric],
        f if f == FIELD_UNIT_PRICE => &[ColumnDataType::Numeric],
        f if f == FIELD_AMOUNT => &[ColumnDataType::Numeric],
        f if f == FIELD_FEE => &[ColumnDataType::Numeric],
        f if f == FIELD_FX_RATE => &[ColumnDataType::Numeric],
        f if f == FIELD_CURRENCY => &[ColumnDataType::Currency],
        f if f == FIELD_ACTIVITY_TYPE => &[ColumnDataType::ActivityType],
        // Account / comment / subtype are free-form — Text is fine.
        f if f == FIELD_ACCOUNT => &[ColumnDataType::Text],
        f if f == FIELD_COMMENT => &[ColumnDataType::Text],
        f if f == FIELD_SUBTYPE => &[ColumnDataType::Text, ColumnDataType::ActivityType],
        _ => &[],
    }
}

/// Per-field header substrings, ordered most-specific-first. These
/// are the same patterns the legacy `auto_detect_field_mappings`
/// uses, copied here so the scorer can run independently. They
/// stay in sync because both modules are tested against the same
/// broker CSV fixtures.
fn header_patterns_for_field(field: &str) -> &'static [&'static str] {
    match field {
        f if f == FIELD_DATE => &[
            "trade date",
            "trade_date",
            "transaction date",
            "transaction_date",
            "activity date",
            "activity_date",
            "settlement date",
            "purchase date",
            "execution date",
            "datetime",
            "date",
            "time",
        ],
        f if f == FIELD_ACTIVITY_TYPE => &[
            "activity type",
            "activity_type",
            "transaction type",
            "transaction_type",
            "trade type",
            "trans type",
            "trans code",
            "action",
            "operation",
            "activity",
            "type",
        ],
        f if f == FIELD_SYMBOL => &[
            "ticker symbol",
            "stock symbol",
            "symbol",
            "ticker",
            "instrument",
            "isin",
            "cusip",
            "security",
            "stock",
            "asset",
        ],
        f if f == FIELD_QUANTITY => &[
            "number of shares",
            "no of shares",
            "no. of shares",
            "share count",
            "shares",
            "quantity",
            "qty",
            "units",
        ],
        f if f == FIELD_UNIT_PRICE => &[
            "purchase price",
            "purchase_price",
            "trade price",
            "tradeprice",
            "execution price",
            "fill price",
            "cost per share",
            "cost basis per share",
            "avg price",
            "average price",
            "unit price",
            "unit_price",
            "share price",
            "share_price",
            "price per share",
            "price",
        ],
        f if f == FIELD_AMOUNT => &[
            "net amount",
            "gross amount",
            "total amount",
            "total value",
            "total cost",
            "trade amount",
            "transaction amount",
            "settlement amount",
            "proceeds",
            "net value",
            "amount",
            "total",
            "market value",
            "current value",
            "value",
            "cost",
        ],
        f if f == FIELD_CURRENCY => &["currency", "ccy", "currency code", "curr", "trade currency"],
        f if f == FIELD_FEE => &[
            "fees & comm",
            "fee",
            "fees",
            "commission",
            "commissions",
            "trading fee",
            "transaction fee",
            "brokerage",
            "charges",
        ],
        f if f == FIELD_ACCOUNT => &[
            "account",
            "account id",
            "account name",
            "portfolio",
            "account number",
        ],
        f if f == FIELD_COMMENT => &[
            "comment",
            "comments",
            "note",
            "notes",
            "description",
            "memo",
            "remarks",
        ],
        f if f == FIELD_FX_RATE => &[
            "fx rate",
            "fxrate",
            "fx_rate",
            "exchange rate",
            "exchangerate",
            "exchange_rate",
            "forex rate",
            "conversion rate",
        ],
        f if f == FIELD_SUBTYPE => &[
            "subtype",
            "sub type",
            "sub_type",
            "variation",
            "subcategory",
        ],
        _ => &[],
    }
}

/// Score a single (column, field) pair on the \[0.0, 1.0\] scale.
///
/// The score is a weighted sum of three independent signals:
///
/// * Header text match (weight 0.5) — exact match scores 1.0;
///   substring match scores by the *specificity rank* of the
///   matched pattern (more-specific-first ordering); no match
///   scores 0.
/// * Data-type match (weight 0.4) — the column's inferred kind
///   either matches one of the field's expected kinds (×
///   `kind_confidence`) or it doesn't (0).
/// * Integer-shape bonus (weight 0.1) — for `quantity` only,
///   integer-valued numeric columns get a small bonus over
///   decimal-valued ones. Lets us prefer "Shares" (123) over
///   "Price" (123.45) when both score equal on text.
///
/// Weights were chosen to keep the legacy header-only signal
/// dominant (so we don't regress existing well-behaved CSVs) while
/// letting data-type evidence break ties and override bad header
/// matches when the evidence is strong.
fn score_pair(field: &str, profile: &ColumnProfile) -> f32 {
    let mut score = 0.0_f32;

    // Header text signal.
    let patterns = header_patterns_for_field(field);
    if !patterns.is_empty() {
        let mut header_score = 0.0_f32;
        for (rank, pattern) in patterns.iter().enumerate() {
            let specificity = 1.0 - (rank as f32 / patterns.len() as f32);
            if profile.header_lower == *pattern {
                header_score = header_score.max(1.0 * specificity.max(0.5));
            } else if profile.header_lower.contains(pattern) {
                header_score = header_score.max(0.75 * specificity.max(0.4));
            }
        }
        score += 0.5 * header_score;
    }

    // Data type signal.
    let expected = expected_kind_for_field(field);
    if !expected.is_empty() && expected.iter().any(|k| *k == profile.kind) {
        score += 0.4 * profile.kind_confidence;
    } else if !expected.is_empty() && profile.kind == ColumnDataType::Empty {
        // Empty column gives no positive evidence, no penalty.
    } else if !expected.is_empty() {
        // Wrong kind with strong confidence — explicitly contradict
        // the header. Eg. header "Price" but column has letters,
        // not numbers → discount this candidate heavily.
        score -= 0.3 * profile.kind_confidence;
    }

    // Integer-shape bonus for quantity.
    if field == FIELD_QUANTITY && profile.looks_integer {
        score += 0.1;
    }

    score
}

/// Greedy assignment: walk every (field, column) candidate by
/// descending score and lock in the top scorer for each field,
/// skipping columns that have already been claimed.
///
/// Returns the same shape as the legacy detector (field → header
/// string), so it's a drop-in replacement when we have data to
/// profile.
fn assign_greedy(profiles: &[ColumnProfile]) -> HashMap<String, String> {
    let fields: &[&str] = &[
        FIELD_DATE,
        FIELD_ACTIVITY_TYPE,
        FIELD_SYMBOL,
        FIELD_QUANTITY,
        FIELD_UNIT_PRICE,
        FIELD_AMOUNT,
        FIELD_CURRENCY,
        FIELD_FEE,
        FIELD_FX_RATE,
        FIELD_ACCOUNT,
        FIELD_COMMENT,
        FIELD_SUBTYPE,
    ];

    // Build all candidates above a minimum threshold so very weak
    // matches don't pollute the assignment. 0.15 was picked
    // empirically against the broker fixture tests — high enough
    // to drop accidental matches, low enough to keep legitimately
    // header-only signals (where data isn't profilable).
    const THRESHOLD: f32 = 0.15;
    let mut candidates: Vec<(f32, &str, usize)> = Vec::new();
    for field in fields {
        for (idx, profile) in profiles.iter().enumerate() {
            let s = score_pair(field, profile);
            if s >= THRESHOLD {
                candidates.push((s, field, idx));
            }
        }
    }

    // Sort descending by score. Ties break by the field's position
    // in the canonical list (earlier fields = more important);
    // remaining ties by column index for determinism.
    candidates.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                fields
                    .iter()
                    .position(|f| *f == a.1)
                    .cmp(&fields.iter().position(|f| *f == b.1))
            })
            .then_with(|| a.2.cmp(&b.2))
    });

    let mut mapping = HashMap::new();
    let mut used_columns: Vec<bool> = vec![false; profiles.len()];
    for (_score, field, idx) in candidates {
        if mapping.contains_key(field) || used_columns[idx] {
            continue;
        }
        mapping.insert(field.to_string(), profiles[idx].header.clone());
        used_columns[idx] = true;
    }
    mapping
}

/// Detect column → field mappings using all available signals.
///
/// Path:
/// 1. Try [`recognise_broker`] — instant match for known CSV formats.
/// 2. If sample rows are available, build [`ColumnProfile`]s and
///    run [`score_pair`] across every column × field combination,
///    assigning greedily.
/// 3. If sample rows are NOT available (e.g. we got headers but
///    couldn't parse the body), fall back to header-only scoring
///    by passing in profiles that have `kind == Empty`. The text
///    signal still does the heavy lifting in that case.
pub fn detect_field_mappings_smart(
    headers: &[String],
    sample_rows: &[Vec<String>],
) -> HashMap<String, String> {
    if let Some((_name, mapping)) = recognise_broker(headers) {
        return mapping;
    }
    let profiles = build_profiles(headers, sample_rows);
    assign_greedy(&profiles)
}

/// Cross-column reconciliation.
///
/// After assignment, validate that `quantity × unit_price ≈ amount`
/// on rows that have all three values populated. Returns the share
/// of valid rows where the relationship holds within 1%.
///
/// `1.0` = every checkable row reconciles; `0.0` = none do; `None`
/// = no rows had all three fields populated, so we couldn't check.
///
/// Use cases:
/// - Show a warning in the preview if reconciliation is low —
///   the user probably has the columns mapped wrong.
/// - Pick between two equally-scored column candidates by which
///   one reconciles better.
pub fn reconcile(
    mapping: &HashMap<String, String>,
    headers: &[String],
    rows: &[Vec<String>],
) -> Option<f32> {
    let qty_col = mapping.get(FIELD_QUANTITY)?;
    let price_col = mapping.get(FIELD_UNIT_PRICE)?;
    let amount_col = mapping.get(FIELD_AMOUNT)?;

    let qty_idx = headers.iter().position(|h| h == qty_col)?;
    let price_idx = headers.iter().position(|h| h == price_col)?;
    let amount_idx = headers.iter().position(|h| h == amount_col)?;

    let mut checked = 0usize;
    let mut matched = 0usize;

    for row in rows {
        let qty = row.get(qty_idx).and_then(|s| parse_loose_number(s));
        let price = row.get(price_idx).and_then(|s| parse_loose_number(s));
        let amount = row.get(amount_idx).and_then(|s| parse_loose_number(s));

        if let (Some(q), Some(p), Some(a)) = (qty, price, amount) {
            if a == 0.0 || (q == 0.0 && p == 0.0) {
                continue;
            }
            checked += 1;
            let expected = q.abs() * p.abs();
            let observed = a.abs();
            let denom = expected.abs().max(1e-9);
            let rel_err = (expected - observed).abs() / denom;
            if rel_err < 0.01 {
                matched += 1;
            }
        }
    }

    if checked == 0 {
        None
    } else {
        Some(matched as f32 / checked as f32)
    }
}

/// Parse a "loose" number — strips currency symbols, thousands
/// separators (both US and European conventions), and parentheses
/// (which some brokers use for negatives). Returns `None` for
/// anything that doesn't parse.
fn parse_loose_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut stripped: String = s
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '$' | '€' | '£' | '¥' | '₹' | '₩' | '₿' | ' ' | '\u{00A0}'
            )
        })
        .collect();
    let negative_paren = stripped.starts_with('(') && stripped.ends_with(')');
    if negative_paren {
        stripped = stripped[1..stripped.len() - 1].to_string();
    }

    // Disambiguate US vs European format from the LAST separator:
    // `1.234,56` → European (last sep is `,` → decimal point);
    // `1,234.56` → US (last sep is `.` → decimal point). Naively
    // stripping commas in US mode turns the European value into
    // `1.23456`, off by three orders of magnitude — silent
    // corruption the reconciler would otherwise catch only
    // statistically.
    let last_dot = stripped.rfind('.');
    let last_comma = stripped.rfind(',');
    let european = match (last_dot, last_comma) {
        (Some(d), Some(c)) => c > d,
        (Some(_), None) => false,
        (None, Some(c)) => {
            // Only commas: could be European decimal ("12,5") or
            // US thousands grouping ("12,500"). Tail length wins.
            let tail = &stripped[c + 1..];
            tail.len() <= 2 && tail.chars().all(|ch| ch.is_ascii_digit())
        }
        (None, None) => false,
    };

    let normalised = if european {
        stripped.replace('.', "").replace(',', ".")
    } else {
        stripped.replace(',', "")
    };

    normalised
        .parse::<f64>()
        .ok()
        .map(|v| if negative_paren { -v.abs() } else { v })
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
        rows.iter()
            .map(|r| r.iter().map(|s| s.to_string()).collect())
            .collect()
    }
    fn hdrs(headers: &[&str]) -> Vec<String> {
        headers.iter().map(|s| s.to_string()).collect()
    }

    // ─────────────────────────────────────────────────────────────
    // Cell classifier
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn classify_cell_dates() {
        for s in &[
            "2024-01-15",
            "01/15/2024",
            "15/01/2024",
            "15.01.2024",
            "2024-01-15 09:30:00",
            "2024-01-15T09:30:00Z",
        ] {
            assert_eq!(
                classify_cell(s),
                ColumnDataType::Date,
                "{} should be Date",
                s
            );
        }
    }

    #[test]
    fn classify_cell_numeric() {
        for s in &[
            "123",
            "123.45",
            "1,234.56",
            "1.234,56",
            "$1,234.56",
            "(500)",
            "-5.5",
            "₹100,000",
        ] {
            assert_eq!(
                classify_cell(s),
                ColumnDataType::Numeric,
                "{} should be Numeric",
                s
            );
        }
    }

    #[test]
    fn classify_cell_currency() {
        for s in &["USD", "EUR", "INR", "$", "₹", "GBP"] {
            assert_eq!(
                classify_cell(s),
                ColumnDataType::Currency,
                "{} should be Currency",
                s
            );
        }
    }

    #[test]
    fn classify_cell_symbol() {
        for s in &["AAPL", "BRK.B", "0700.HK", "RELIANCE.NS", "BTC/USD", "VOO"] {
            assert_eq!(
                classify_cell(s),
                ColumnDataType::Symbol,
                "{} should be Symbol",
                s
            );
        }
    }

    #[test]
    fn classify_cell_activity_type() {
        for s in &[
            "BUY",
            "Sell",
            "DIVIDEND",
            "Div",
            "FEE",
            "interest",
            "Deposit",
            "Withdrawal",
        ] {
            assert_eq!(
                classify_cell(s),
                ColumnDataType::ActivityType,
                "{} should be ActivityType",
                s
            );
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Column profile
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn profile_distinguishes_quantity_from_price() {
        // Quantity = integers; price = decimals. Profile should
        // mark quantity as integer-shaped, price as not.
        let qty =
            ColumnProfile::infer("Qty", &["10".into(), "25".into(), "100".into(), "5".into()]);
        let price = ColumnProfile::infer(
            "Price",
            &["100.50".into(), "250.75".into(), "1,234.50".into()],
        );
        assert!(qty.looks_integer, "quantity column should look integer");
        assert!(!price.looks_integer, "price column should NOT look integer");
        assert_eq!(qty.kind, ColumnDataType::Numeric);
        assert_eq!(price.kind, ColumnDataType::Numeric);
    }

    #[test]
    fn profile_picks_dominant_type_with_some_noise() {
        // 4/5 rows are numeric, 1 row is empty (filtered) — still
        // confidently numeric.
        let p = ColumnProfile::infer(
            "Amount",
            &[
                "100".into(),
                "200".into(),
                "".into(),
                "300".into(),
                "400".into(),
            ],
        );
        assert_eq!(p.kind, ColumnDataType::Numeric);
        assert!(p.kind_confidence >= 0.99, "got {}", p.kind_confidence);
        assert_eq!(p.samples_seen, 4);
    }

    #[test]
    fn profile_empty_column() {
        let p = ColumnProfile::infer("Notes", &["".into(), " ".into()]);
        assert_eq!(p.kind, ColumnDataType::Empty);
    }

    // ─────────────────────────────────────────────────────────────
    // Broker fingerprinting
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn recognise_yahoo_portfolio() {
        let headers = hdrs(&[
            "Symbol",
            "Current Price",
            "Date",
            "Time",
            "Change",
            "Open",
            "High",
            "Low",
            "Volume",
            "Trade Date",
            "Purchase Price",
            "Shares",
            "Commission",
            "Comment",
        ]);
        let m = recognise_broker(&headers).expect("Yahoo should match");
        assert_eq!(m.0, "Yahoo Finance Portfolio");
        assert_eq!(m.1.get(FIELD_DATE), Some(&"Trade Date".to_string()));
        assert_eq!(
            m.1.get(FIELD_UNIT_PRICE),
            Some(&"Purchase Price".to_string())
        );
        assert_eq!(m.1.get(FIELD_QUANTITY), Some(&"Shares".to_string()));
        assert_eq!(m.1.get(FIELD_SYMBOL), Some(&"Symbol".to_string()));
    }

    #[test]
    fn recognise_zerodha() {
        let headers = hdrs(&[
            "Symbol",
            "ISIN",
            "Trade Date",
            "Exchange",
            "Segment",
            "Series",
            "Trade Type",
            "Auction",
            "Quantity",
            "Price",
            "Trade ID",
            "Order ID",
            "Order Execution Time",
            "Brokerage",
            "STT",
            "GST",
            "Stamp Duty",
        ]);
        let m = recognise_broker(&headers).expect("Zerodha should match");
        assert_eq!(m.0, "Zerodha Tradebook");
        assert_eq!(m.1.get(FIELD_DATE), Some(&"Trade Date".to_string()));
        assert_eq!(m.1.get(FIELD_QUANTITY), Some(&"Quantity".to_string()));
        assert_eq!(m.1.get(FIELD_UNIT_PRICE), Some(&"Price".to_string()));
        assert_eq!(
            m.1.get(FIELD_ACTIVITY_TYPE),
            Some(&"Trade Type".to_string())
        );
    }

    #[test]
    fn recognise_unknown_returns_none() {
        let headers = hdrs(&["Foo", "Bar", "Baz"]);
        assert!(recognise_broker(&headers).is_none());
    }

    // ─────────────────────────────────────────────────────────────
    // End-to-end smart detection
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn smart_detect_yahoo_picks_trade_columns_via_fingerprint() {
        let headers = hdrs(&[
            "Symbol",
            "Current Price",
            "Date",
            "Trade Date",
            "Purchase Price",
            "Shares",
            "Commission",
        ]);
        let m = detect_field_mappings_smart(&headers, &[]);
        // Fingerprint path — should pick the trade columns even
        // with zero sample data, because the fingerprint is
        // header-signature only.
        assert_eq!(m.get(FIELD_UNIT_PRICE), Some(&"Purchase Price".to_string()));
        assert_eq!(m.get(FIELD_DATE), Some(&"Trade Date".to_string()));
        assert_eq!(m.get(FIELD_QUANTITY), Some(&"Shares".to_string()));
    }

    #[test]
    fn smart_detect_data_aware_overrides_misleading_header() {
        // Adversarial: column is HEADERED "Price" but actually
        // contains symbols. The legacy header-only matcher would
        // map "Price" → FIELD_UNIT_PRICE and produce wrong amounts.
        // The data-aware scorer should refuse, recognise the
        // symbol-shape, and route it to FIELD_SYMBOL instead.
        let headers = hdrs(&["Date", "Price", "Quantity", "Unit Price"]);
        let sample = rows(&[
            &["2024-01-15", "AAPL", "10", "150.00"],
            &["2024-01-16", "MSFT", "5", "200.00"],
            &["2024-01-17", "GOOG", "8", "125.00"],
        ]);
        let m = detect_field_mappings_smart(&headers, &sample);

        // Killer assertion: "Price" must NOT be picked as the
        // unit_price field, because its data is symbols. This is
        // the bug uncle's Yahoo CSV import hit.
        assert_ne!(
            m.get(FIELD_UNIT_PRICE),
            Some(&"Price".to_string()),
            "data-aware scorer must reject the misleading 'Price' header"
        );
        // The symbol-bearing column ends up as FIELD_SYMBOL.
        assert_eq!(m.get(FIELD_SYMBOL), Some(&"Price".to_string()));
        // "Unit Price" claims FIELD_UNIT_PRICE on a strong header
        // match — that's what we want a real price column to do.
        assert_eq!(m.get(FIELD_UNIT_PRICE), Some(&"Unit Price".to_string()));
        assert_eq!(m.get(FIELD_QUANTITY), Some(&"Quantity".to_string()));
        assert_eq!(m.get(FIELD_DATE), Some(&"Date".to_string()));
    }

    #[test]
    fn smart_detect_no_double_claim() {
        let headers = hdrs(&["Date", "Symbol", "Shares", "Price", "Total"]);
        let sample = rows(&[
            &["2024-01-15", "AAPL", "10", "150.00", "1500.00"],
            &["2024-01-16", "MSFT", "5", "200.00", "1000.00"],
        ]);
        let m = detect_field_mappings_smart(&headers, &sample);
        // Every claimed header is unique.
        let claimed: std::collections::HashSet<_> = m.values().cloned().collect();
        assert_eq!(claimed.len(), m.len());
        // And the obvious mappings are right.
        assert_eq!(m.get(FIELD_QUANTITY), Some(&"Shares".to_string()));
        assert_eq!(m.get(FIELD_UNIT_PRICE), Some(&"Price".to_string()));
        assert_eq!(m.get(FIELD_AMOUNT), Some(&"Total".to_string()));
    }

    // ─────────────────────────────────────────────────────────────
    // Reconciliation
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn reconcile_perfect_match() {
        let headers = hdrs(&["Date", "Qty", "Price", "Total"]);
        let sample = rows(&[
            &["2024-01-15", "10", "150.00", "1500.00"],
            &["2024-01-16", "5", "200.00", "1000.00"],
            &["2024-01-17", "8", "125.00", "1000.00"],
        ]);
        let mut m = HashMap::new();
        m.insert(FIELD_QUANTITY.to_string(), "Qty".to_string());
        m.insert(FIELD_UNIT_PRICE.to_string(), "Price".to_string());
        m.insert(FIELD_AMOUNT.to_string(), "Total".to_string());
        let r = reconcile(&m, &headers, &sample).expect("should reconcile");
        assert!(r > 0.99, "got {}", r);
    }

    #[test]
    fn reconcile_with_currency_symbols_in_amount() {
        let headers = hdrs(&["Qty", "Price", "Amount"]);
        let sample = rows(&[
            &["10", "150.00", "$1,500.00"],
            &["5", "200.00", "$1,000.00"],
        ]);
        let mut m = HashMap::new();
        m.insert(FIELD_QUANTITY.to_string(), "Qty".to_string());
        m.insert(FIELD_UNIT_PRICE.to_string(), "Price".to_string());
        m.insert(FIELD_AMOUNT.to_string(), "Amount".to_string());
        let r = reconcile(&m, &headers, &sample).expect("should reconcile");
        assert!(r > 0.99, "got {}", r);
    }

    #[test]
    fn reconcile_picks_up_mismatch() {
        // Amount column doesn't reconcile — should score 0.
        let headers = hdrs(&["Qty", "Price", "Amount"]);
        let sample = rows(&[&["10", "150.00", "999.99"], &["5", "200.00", "42.00"]]);
        let mut m = HashMap::new();
        m.insert(FIELD_QUANTITY.to_string(), "Qty".to_string());
        m.insert(FIELD_UNIT_PRICE.to_string(), "Price".to_string());
        m.insert(FIELD_AMOUNT.to_string(), "Amount".to_string());
        let r = reconcile(&m, &headers, &sample).expect("should reconcile");
        assert!(r < 0.01, "got {}", r);
    }

    #[test]
    fn reconcile_no_overlap_returns_none() {
        // Rows missing at least one of qty/price/amount everywhere.
        let headers = hdrs(&["Qty", "Price", "Amount"]);
        let sample = rows(&[&["10", "", "1500.00"], &["", "200.00", "1000.00"]]);
        let mut m = HashMap::new();
        m.insert(FIELD_QUANTITY.to_string(), "Qty".to_string());
        m.insert(FIELD_UNIT_PRICE.to_string(), "Price".to_string());
        m.insert(FIELD_AMOUNT.to_string(), "Amount".to_string());
        assert!(reconcile(&m, &headers, &sample).is_none());
    }

    // ─────────────────────────────────────────────────────────────
    // Misc invariants
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_loose_number_handles_european_format() {
        assert_eq!(parse_loose_number("1.234,56"), Some(1234.56));
        assert_eq!(parse_loose_number("1,234.56"), Some(1234.56));
        assert_eq!(parse_loose_number("(500)"), Some(-500.0));
        assert_eq!(parse_loose_number("₹100,000"), Some(100_000.0));
        assert_eq!(parse_loose_number("$1,500.00"), Some(1500.00));
        assert_eq!(parse_loose_number(""), None);
        assert_eq!(parse_loose_number("AAPL"), None);
    }
}
