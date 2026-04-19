//! Generates hledger journal text from structured manual entry data.

use serde::{Deserialize, Serialize};

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Domain types (mirrored in frontend/src/types/manual.ts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Posting {
    pub account: String,
    pub amount: Option<String>,
    pub commodity: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceEntry {
    pub date: String,
    pub commodity: String,
    pub amount: String,
    pub target_commodity: String,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEntry {
    pub date: String,
    pub status: String,
    pub code: String,
    pub description: String,
    pub comment: String,
    pub postings: Vec<Posting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodicEntry {
    pub period: String,
    pub description: String,
    pub comment: String,
    pub postings: Vec<Posting>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

static DATE_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());

static AMOUNT_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^-?\d+(\.\d+)?$").unwrap());

fn validate_date(date: &str, ctx: &str) -> Result<(), AppError> {
    if !DATE_RE.is_match(date) {
        return Err(AppError::BadRequest(format!(
            "{ctx}: invalid date '{date}'; expected YYYY-MM-DD"
        )));
    }
    Ok(())
}

fn validate_amount(amount: &str, ctx: &str) -> Result<(), AppError> {
    if !AMOUNT_RE.is_match(amount) {
        return Err(AppError::BadRequest(format!(
            "{ctx}: invalid amount '{amount}'; expected a decimal number"
        )));
    }
    Ok(())
}

fn validate_nonempty(value: &str, field: &str, ctx: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::BadRequest(format!(
            "{ctx}: {field} cannot be empty"
        )));
    }
    Ok(())
}

fn validate_postings(postings: &[Posting], ctx: &str) -> Result<(), AppError> {
    if postings.is_empty() {
        return Err(AppError::BadRequest(format!(
            "{ctx}: must have at least one posting"
        )));
    }
    for (i, p) in postings.iter().enumerate() {
        validate_nonempty(&p.account, "account", &format!("{ctx} posting #{}", i + 1))?;
        if let Some(ref amt) = p.amount
            && !amt.is_empty()
        {
            validate_amount(amt, &format!("{ctx} posting #{}", i + 1))?;
        }
    }
    Ok(())
}

pub fn validate_price(p: &PriceEntry) -> Result<(), AppError> {
    validate_date(&p.date, "price")?;
    validate_nonempty(&p.commodity, "commodity", "price")?;
    validate_amount(&p.amount, "price")?;
    validate_nonempty(&p.target_commodity, "target commodity", "price")?;
    Ok(())
}

pub fn validate_transaction(t: &TransactionEntry) -> Result<(), AppError> {
    validate_date(&t.date, "transaction")?;
    validate_nonempty(&t.description, "description", "transaction")?;
    validate_postings(&t.postings, "transaction")?;
    Ok(())
}

pub fn validate_periodic(p: &PeriodicEntry) -> Result<(), AppError> {
    validate_nonempty(&p.period, "period", "periodic transaction")?;
    validate_postings(&p.postings, "periodic transaction")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

fn write_postings(out: &mut String, postings: &[Posting]) {
    for p in postings {
        match (&p.amount, &p.commodity) {
            (Some(amt), Some(comm)) if !amt.is_empty() && !comm.is_empty() => {
                out.push_str(&format!("    {}  {} {}", p.account, amt, comm));
            }
            (Some(amt), _) if !amt.is_empty() => {
                out.push_str(&format!("    {}  {}", p.account, amt));
            }
            _ => {
                out.push_str(&format!("    {}", p.account));
            }
        }
        if let Some(ref c) = p.comment
            && !c.is_empty()
        {
            out.push_str(&format!("  ; {c}"));
        }
        out.push('\n');
    }
}

pub fn generate_journal_text(
    prices: &[PriceEntry],
    transactions: &[TransactionEntry],
    periodics: &[PeriodicEntry],
) -> Result<String, AppError> {
    let mut out = String::new();

    if !prices.is_empty() {
        out.push_str("; --- Prices ---\n");
        for p in prices {
            if !p.comment.is_empty() {
                out.push_str(&format!("; {}\n", p.comment));
            }
            out.push_str(&format!(
                "P {} {} {} {}\n",
                p.date, p.commodity, p.amount, p.target_commodity
            ));
        }
    }

    if !periodics.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("; --- Periodic transactions (budgets) ---\n");
        for p in periodics {
            let desc = if p.description.is_empty() {
                String::new()
            } else {
                format!("  {}", p.description)
            };
            out.push_str(&format!("~ {}{}\n", p.period, desc));
            if !p.comment.is_empty() {
                out.push_str(&format!("    ; {}\n", p.comment));
            }
            write_postings(&mut out, &p.postings);
        }
    }

    if !transactions.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("; --- Manual transactions ---\n");
        for t in transactions {
            // First line: DATE [STATUS] [(CODE)] DESCRIPTION
            let mut first = t.date.clone();
            if !t.status.is_empty() {
                first.push(' ');
                first.push_str(&t.status);
            }
            if !t.code.is_empty() {
                first.push_str(&format!(" ({})", t.code));
            }
            first.push(' ');
            first.push_str(&t.description);
            out.push_str(&first);
            out.push('\n');
            if !t.comment.is_empty() {
                out.push_str(&format!("    ; {}\n", t.comment));
            }
            write_postings(&mut out, &t.postings);
        }
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn price(date: &str, commodity: &str, amount: &str, target: &str) -> PriceEntry {
        PriceEntry {
            date: date.into(),
            commodity: commodity.into(),
            amount: amount.into(),
            target_commodity: target.into(),
            comment: String::new(),
        }
    }

    fn posting(account: &str, amount: Option<&str>, commodity: Option<&str>) -> Posting {
        Posting {
            account: account.into(),
            amount: amount.map(String::from),
            commodity: commodity.map(String::from),
            comment: None,
        }
    }

    fn txn(date: &str, desc: &str, postings: Vec<Posting>) -> TransactionEntry {
        TransactionEntry {
            date: date.into(),
            status: String::new(),
            code: String::new(),
            description: desc.into(),
            comment: String::new(),
            postings,
        }
    }

    fn periodic(period: &str, desc: &str, postings: Vec<Posting>) -> PeriodicEntry {
        PeriodicEntry {
            period: period.into(),
            description: desc.into(),
            comment: String::new(),
            postings,
        }
    }

    #[test]
    fn empty_produces_empty_output() {
        let text = generate_journal_text(&[], &[], &[]).unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn single_price_directive() {
        let prices = vec![price("2026-04-15", "AAPL", "170.50", "USD")];
        let text = generate_journal_text(&prices, &[], &[]).unwrap();
        assert!(
            text.contains("P 2026-04-15 AAPL 170.50 USD\n"),
            "got: {text}"
        );
    }

    #[test]
    fn multiple_prices_in_order() {
        let prices = vec![
            price("2026-04-10", "EUR", "1.08", "USD"),
            price("2026-04-15", "AAPL", "170.50", "USD"),
        ];
        let text = generate_journal_text(&prices, &[], &[]).unwrap();
        let pos_eur = text.find("EUR").unwrap();
        let pos_aapl = text.find("AAPL").unwrap();
        assert!(pos_eur < pos_aapl);
    }

    #[test]
    fn transaction_with_two_postings() {
        let t = txn(
            "2026-04-18",
            "Groceries",
            vec![
                posting("expenses:groceries", Some("50.00"), Some("USD")),
                posting("assets:checking", None, None),
            ],
        );
        let text = generate_journal_text(&[], &[t], &[]).unwrap();
        assert!(text.contains("2026-04-18 Groceries\n"), "got: {text}");
        assert!(text.contains("    expenses:groceries  50.00 USD\n"));
        assert!(text.contains("    assets:checking\n"));
    }

    #[test]
    fn cleared_transaction_status() {
        let mut t = txn(
            "2026-01-01",
            "Salary",
            vec![
                posting("income:salary", Some("3000"), Some("USD")),
                posting("assets:checking", None, None),
            ],
        );
        t.status = "*".into();
        let text = generate_journal_text(&[], &[t], &[]).unwrap();
        assert!(text.contains("2026-01-01 * Salary\n"), "got: {text}");
    }

    #[test]
    fn periodic_transaction() {
        let p = periodic(
            "monthly",
            "Groceries budget",
            vec![
                posting("expenses:groceries", Some("500"), Some("USD")),
                posting("assets:budget", None, None),
            ],
        );
        let text = generate_journal_text(&[], &[], &[p]).unwrap();
        assert!(
            text.contains("~ monthly  Groceries budget\n"),
            "got: {text}"
        );
        assert!(text.contains("    expenses:groceries  500 USD\n"));
    }

    #[test]
    fn sections_ordered_prices_periodics_transactions() {
        let prices = vec![price("2026-01-01", "EUR", "1.08", "USD")];
        let txns = vec![txn(
            "2026-01-02",
            "Misc",
            vec![
                posting("expenses:misc", Some("10"), Some("USD")),
                posting("assets:checking", None, None),
            ],
        )];
        let periodics = vec![periodic(
            "monthly",
            "Budget",
            vec![
                posting("expenses:food", Some("200"), Some("USD")),
                posting("assets:budget", None, None),
            ],
        )];
        let text = generate_journal_text(&prices, &txns, &periodics).unwrap();
        let pos_prices = text.find("Prices").unwrap();
        let pos_periodics = text.find("Periodic").unwrap();
        let pos_transactions = text.find("Manual transactions").unwrap();
        assert!(pos_prices < pos_periodics);
        assert!(pos_periodics < pos_transactions);
    }

    #[test]
    fn validate_bad_date() {
        let p = price("not-a-date", "EUR", "1.0", "USD");
        assert!(validate_price(&p).is_err());
    }

    #[test]
    fn validate_bad_amount() {
        let p = price("2026-01-01", "EUR", "abc", "USD");
        assert!(validate_price(&p).is_err());
    }

    #[test]
    fn validate_empty_commodity() {
        let p = price("2026-01-01", "", "1.0", "USD");
        assert!(validate_price(&p).is_err());
    }

    #[test]
    fn validate_empty_postings() {
        let t = txn("2026-01-01", "Test", vec![]);
        assert!(validate_transaction(&t).is_err());
    }

    #[test]
    fn validate_empty_period() {
        let p = periodic(
            "",
            "Budget",
            vec![posting("expenses:food", Some("100"), Some("USD"))],
        );
        assert!(validate_periodic(&p).is_err());
    }

    #[test]
    fn validate_good_transaction() {
        let t = txn(
            "2026-04-01",
            "Test",
            vec![
                posting("expenses:food", Some("10.00"), Some("USD")),
                posting("assets:cash", None, None),
            ],
        );
        assert!(validate_transaction(&t).is_ok());
    }
}
