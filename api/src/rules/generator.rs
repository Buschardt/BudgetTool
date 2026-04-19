//! Generates hledger `.rules` file text from a structured `RulesConfig`.

use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

// ---------------------------------------------------------------------------
// Config types (mirrored in the frontend as TypeScript interfaces)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RulesConfig {
    pub skip: Option<u32>,
    pub separator: Option<Separator>,
    pub date_format: Option<String>,
    pub decimal_mark: Option<DecimalMark>,
    pub newest_first: Option<bool>,
    pub intra_day_reversed: Option<bool>,
    pub balance_type: Option<String>,
    pub encoding: Option<String>,
    pub timezone: Option<String>,

    /// Names for each CSV column in order.  Empty string means unnamed/ignored.
    #[serde(default)]
    pub fields: Vec<String>,

    /// Top-level field assignments (outside any conditional).
    #[serde(default)]
    pub assignments: Vec<FieldAssignment>,

    /// Ordered list of conditional rules.
    #[serde(default)]
    pub conditionals: Vec<ConditionalRule>,

    /// IDs of other rules configs to include (resolved to disk paths at write time).
    #[serde(default)]
    pub includes: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Separator {
    Comma,
    Semicolon,
    Tab,
    Space,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecimalMark {
    #[serde(rename = ".")]
    Dot,
    #[serde(rename = ",")]
    Comma,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldAssignment {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalRule {
    #[serde(rename = "type")]
    pub rule_type: ConditionalType,

    // Block-type fields
    #[serde(default)]
    pub match_groups: Vec<MatchGroup>,
    #[serde(default)]
    pub assignments: Vec<FieldAssignment>,
    #[serde(default)]
    pub skip: bool,
    #[serde(default)]
    pub end: bool,

    // Table-type fields
    #[serde(default)]
    pub table_fields: Vec<String>,
    #[serde(default)]
    pub table_rows: Vec<TableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionalType {
    Block,
    Table,
}

/// A group of matchers that are AND-combined.
/// Multiple MatchGroups in a ConditionalRule are OR-combined.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchGroup {
    #[serde(default)]
    pub matchers: Vec<Matcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matcher {
    pub pattern: String,
    /// If set, restricts matching to this named CSV field (emits `%fieldname` prefix).
    pub field: Option<String>,
    #[serde(default)]
    pub negate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub pattern: String,
    #[serde(default)]
    pub values: Vec<String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Valid hledger field names that can be assigned to.
const VALID_HLEDGER_FIELDS: &[&str] = &[
    "date",
    "date2",
    "status",
    "code",
    "description",
    "comment",
    "account1",
    "account2",
    "account3",
    "account4",
    "account5",
    "account6",
    "account7",
    "account8",
    "account9",
    "account10",
    "amount",
    "amount-in",
    "amount-out",
    "amount1",
    "amount2",
    "amount3",
    "amount4",
    "amount5",
    "amount6",
    "amount7",
    "amount8",
    "amount9",
    "amount10",
    "amount1-in",
    "amount1-out",
    "amount2-in",
    "amount2-out",
    "amount3-in",
    "amount3-out",
    "currency",
    "currency1",
    "currency2",
    "currency3",
    "balance",
    "balance1",
    "balance2",
    "balance3",
    "comment1",
    "comment2",
    "comment3",
];

fn validate_field_name(name: &str) -> Result<(), AppError> {
    if VALID_HLEDGER_FIELDS.contains(&name) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "invalid hledger field name: '{name}'"
        )))
    }
}

fn validate_regex(pattern: &str) -> Result<(), AppError> {
    regex::Regex::new(pattern)
        .map_err(|e| AppError::BadRequest(format!("invalid regex pattern '{pattern}': {e}")))?;
    Ok(())
}

fn validate_balance_type(bt: &str) -> Result<(), AppError> {
    match bt {
        "=" | "=*" | "==" | "==*" => Ok(()),
        _ => Err(AppError::BadRequest(format!(
            "invalid balance-type '{bt}'; must be one of: =, =*, ==, ==*"
        ))),
    }
}

pub fn validate(config: &RulesConfig) -> Result<(), AppError> {
    // Validate balance-type if set
    if let Some(ref bt) = config.balance_type {
        validate_balance_type(bt)?;
    }

    // Validate top-level assignments
    for assignment in &config.assignments {
        validate_field_name(&assignment.field)?;
    }

    // Validate conditionals
    for (i, cond) in config.conditionals.iter().enumerate() {
        match cond.rule_type {
            ConditionalType::Block => {
                for group in &cond.match_groups {
                    for matcher in &group.matchers {
                        validate_regex(&matcher.pattern).map_err(|e| match e {
                            AppError::BadRequest(msg) => {
                                AppError::BadRequest(format!("conditional #{}: {msg}", i + 1))
                            }
                            other => other,
                        })?;
                    }
                }
                for assignment in &cond.assignments {
                    validate_field_name(&assignment.field).map_err(|e| match e {
                        AppError::BadRequest(msg) => {
                            AppError::BadRequest(format!("conditional #{}: {msg}", i + 1))
                        }
                        other => other,
                    })?;
                }
            }
            ConditionalType::Table => {
                for field in &cond.table_fields {
                    if !field.is_empty() {
                        validate_field_name(field).map_err(|e| match e {
                            AppError::BadRequest(msg) => AppError::BadRequest(format!(
                                "conditional #{} (table): {msg}",
                                i + 1
                            )),
                            other => other,
                        })?;
                    }
                }
                for row in &cond.table_rows {
                    validate_regex(&row.pattern).map_err(|e| match e {
                        AppError::BadRequest(msg) => AppError::BadRequest(format!(
                            "conditional #{} (table row): {msg}",
                            i + 1
                        )),
                        other => other,
                    })?;
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generate the text of a `.rules` file from the given config.
/// `include_paths` maps rules config IDs to their disk paths for `include` directives.
pub fn generate_rules_text(
    config: &RulesConfig,
    include_paths: &[(i64, String)],
) -> Result<String, AppError> {
    validate(config)?;

    let mut out = String::new();

    // --- General directives ---
    if let Some(n) = config.skip {
        out.push_str(&format!("skip {n}\n"));
    }
    if let Some(ref sep) = config.separator {
        let s = match sep {
            Separator::Comma => "comma",
            Separator::Semicolon => "semicolon",
            Separator::Tab => "TAB",
            Separator::Space => "SPACE",
        };
        out.push_str(&format!("separator {s}\n"));
    }
    if let Some(ref fmt) = config.date_format {
        out.push_str(&format!("date-format {fmt}\n"));
    }
    if let Some(ref dm) = config.decimal_mark {
        let c = match dm {
            DecimalMark::Dot => ".",
            DecimalMark::Comma => ",",
        };
        out.push_str(&format!("decimal-mark {c}\n"));
    }
    if config.newest_first == Some(true) {
        out.push_str("newest-first\n");
    }
    if config.intra_day_reversed == Some(true) {
        out.push_str("intra-day-reversed\n");
    }
    if let Some(ref bt) = config.balance_type {
        out.push_str(&format!("balance-type {bt}\n"));
    }
    if let Some(ref enc) = config.encoding {
        out.push_str(&format!("encoding {enc}\n"));
    }
    if let Some(ref tz) = config.timezone {
        out.push_str(&format!("timezone {tz}\n"));
    }

    // --- Fields directive ---
    if !config.fields.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("fields ");
        out.push_str(&config.fields.join(", "));
        out.push('\n');
    }

    // --- Top-level assignments ---
    if !config.assignments.is_empty() {
        out.push('\n');
        for a in &config.assignments {
            out.push_str(&format!("{} {}\n", a.field, a.value));
        }
    }

    // --- Conditionals ---
    for cond in &config.conditionals {
        out.push('\n');
        match cond.rule_type {
            ConditionalType::Block => write_block_conditional(&mut out, cond),
            ConditionalType::Table => write_table_conditional(&mut out, cond),
        }
    }

    // --- Includes ---
    for id in &config.includes {
        if let Some((_, path)) = include_paths.iter().find(|(pid, _)| pid == id) {
            out.push('\n');
            out.push_str(&format!("include {path}\n"));
        }
    }

    Ok(out)
}

fn matcher_line(m: &Matcher, and_prefix: bool) -> String {
    let negate = if m.negate { "! " } else { "" };
    let field_prefix = match &m.field {
        Some(f) if !f.is_empty() => format!("%{f} "),
        _ => String::new(),
    };
    if and_prefix {
        format!("& {negate}{field_prefix}{}", m.pattern)
    } else {
        format!("{negate}{field_prefix}{}", m.pattern)
    }
}

fn write_block_conditional(out: &mut String, cond: &ConditionalRule) {
    // Collect non-empty groups
    let groups: Vec<&MatchGroup> = cond
        .match_groups
        .iter()
        .filter(|g| !g.matchers.is_empty())
        .collect();

    if groups.is_empty() {
        return;
    }

    // Decide format:
    //
    // Single group, single matcher:  `if PATTERN\n`
    // Single group, multi matchers:  `if PATTERN\n& PATTERN2\n...`
    // Multiple groups (OR):          `if\nPATTERN_A\n& PATTERN_A2\nPATTERN_B\n`
    if groups.len() == 1 {
        let group = groups[0];
        let first = matcher_line(&group.matchers[0], false);
        out.push_str(&format!("if {first}\n"));
        for m in &group.matchers[1..] {
            out.push_str(&format!("{}\n", matcher_line(m, true)));
        }
    } else {
        out.push_str("if\n");
        for group in &groups {
            let mut first_in_group = true;
            for m in &group.matchers {
                out.push_str(&format!("{}\n", matcher_line(m, !first_in_group)));
                first_in_group = false;
            }
        }
    }

    // Assignments
    for a in &cond.assignments {
        out.push_str(&format!("  {} {}\n", a.field, a.value));
    }
    if cond.skip {
        out.push_str("  skip\n");
    }
    if cond.end {
        out.push_str("  end\n");
    }
}

fn write_table_conditional(out: &mut String, cond: &ConditionalRule) {
    if cond.table_fields.is_empty() || cond.table_rows.is_empty() {
        return;
    }

    // Header: `if,field1,field2,...`
    let header = std::iter::once("if")
        .chain(cond.table_fields.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(",");
    out.push_str(&header);
    out.push('\n');

    for row in &cond.table_rows {
        let mut cells: Vec<String> = vec![row.pattern.clone()];
        let n_fields = cond.table_fields.len();
        for i in 0..n_fields {
            cells.push(row.values.get(i).cloned().unwrap_or_default());
        }
        out.push_str(&cells.join(","));
        out.push('\n');
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn generate(config: &RulesConfig) -> String {
        generate_rules_text(config, &[]).expect("generation failed")
    }

    #[test]
    fn empty_config_produces_empty_output() {
        let config = RulesConfig::default();
        assert_eq!(generate(&config), "");
    }

    #[test]
    fn skip_directive() {
        let config = RulesConfig {
            skip: Some(2),
            ..Default::default()
        };
        assert!(generate(&config).contains("skip 2\n"));
    }

    #[test]
    fn separator_directives() {
        let cases = [
            (Separator::Comma, "separator comma"),
            (Separator::Semicolon, "separator semicolon"),
            (Separator::Tab, "separator TAB"),
            (Separator::Space, "separator SPACE"),
        ];
        for (sep, expected) in cases {
            let config = RulesConfig {
                separator: Some(sep),
                ..Default::default()
            };
            assert!(
                generate(&config).contains(expected),
                "expected '{expected}'"
            );
        }
    }

    #[test]
    fn date_format_directive() {
        let config = RulesConfig {
            date_format: Some("%d/%m/%Y".into()),
            ..Default::default()
        };
        assert!(generate(&config).contains("date-format %d/%m/%Y\n"));
    }

    #[test]
    fn decimal_mark_directive() {
        let config = RulesConfig {
            decimal_mark: Some(DecimalMark::Comma),
            ..Default::default()
        };
        assert!(generate(&config).contains("decimal-mark ,\n"));
    }

    #[test]
    fn newest_first_directive() {
        let config = RulesConfig {
            newest_first: Some(true),
            ..Default::default()
        };
        assert!(generate(&config).contains("newest-first\n"));
    }

    #[test]
    fn balance_type_directive() {
        let config = RulesConfig {
            balance_type: Some("==".into()),
            ..Default::default()
        };
        assert!(generate(&config).contains("balance-type ==\n"));
    }

    #[test]
    fn invalid_balance_type_is_rejected() {
        let config = RulesConfig {
            balance_type: Some("bad".into()),
            ..Default::default()
        };
        assert!(generate_rules_text(&config, &[]).is_err());
    }

    #[test]
    fn fields_directive() {
        let config = RulesConfig {
            fields: vec![
                "date".into(),
                "".into(),
                "description".into(),
                "amount".into(),
            ],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("fields date, , description, amount\n"));
    }

    #[test]
    fn top_level_assignments() {
        let config = RulesConfig {
            assignments: vec![
                FieldAssignment {
                    field: "account1".into(),
                    value: "assets:bank".into(),
                },
                FieldAssignment {
                    field: "description".into(),
                    value: "%3".into(),
                },
            ],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("account1 assets:bank\n"));
        assert!(text.contains("description %3\n"));
    }

    #[test]
    fn invalid_field_name_is_rejected() {
        let config = RulesConfig {
            assignments: vec![FieldAssignment {
                field: "notafield".into(),
                value: "foo".into(),
            }],
            ..Default::default()
        };
        assert!(generate_rules_text(&config, &[]).is_err());
    }

    #[test]
    fn single_matcher_block_conditional() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![Matcher {
                        pattern: "groceries".into(),
                        field: None,
                        negate: false,
                    }],
                }],
                assignments: vec![FieldAssignment {
                    field: "account2".into(),
                    value: "expenses:groceries".into(),
                }],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("if groceries\n"));
        assert!(text.contains("  account2 expenses:groceries\n"));
    }

    #[test]
    fn multi_group_or_conditional() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![
                    MatchGroup {
                        matchers: vec![Matcher {
                            pattern: "amazon".into(),
                            field: None,
                            negate: false,
                        }],
                    },
                    MatchGroup {
                        matchers: vec![Matcher {
                            pattern: "ebay".into(),
                            field: None,
                            negate: false,
                        }],
                    },
                ],
                assignments: vec![FieldAssignment {
                    field: "account2".into(),
                    value: "expenses:shopping".into(),
                }],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("if\n"));
        assert!(text.contains("amazon\n"));
        assert!(text.contains("ebay\n"));
    }

    #[test]
    fn and_matchers_within_group() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![
                        Matcher {
                            pattern: "amazon".into(),
                            field: None,
                            negate: false,
                        },
                        Matcher {
                            pattern: "100\\.".into(),
                            field: Some("amount".into()),
                            negate: false,
                        },
                    ],
                }],
                assignments: vec![FieldAssignment {
                    field: "account2".into(),
                    value: "expenses:shopping".into(),
                }],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("if amazon\n"));
        assert!(text.contains("& %amount 100\\.\n"));
    }

    #[test]
    fn negated_matcher() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![Matcher {
                        pattern: "transfer".into(),
                        field: None,
                        negate: true,
                    }],
                }],
                assignments: vec![FieldAssignment {
                    field: "account2".into(),
                    value: "expenses:misc".into(),
                }],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("if ! transfer\n"));
    }

    #[test]
    fn field_specific_matcher() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![Matcher {
                        pattern: "salary".into(),
                        field: Some("description".into()),
                        negate: false,
                    }],
                }],
                assignments: vec![FieldAssignment {
                    field: "account2".into(),
                    value: "income:salary".into(),
                }],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("if %description salary\n"));
    }

    #[test]
    fn skip_inside_conditional() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![Matcher {
                        pattern: "OPENING BALANCE".into(),
                        field: None,
                        negate: false,
                    }],
                }],
                assignments: vec![],
                skip: true,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("  skip\n"));
    }

    #[test]
    fn table_conditional() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Table,
                match_groups: vec![],
                assignments: vec![],
                skip: false,
                end: false,
                table_fields: vec!["account2".into(), "comment".into()],
                table_rows: vec![
                    TableRow {
                        pattern: "groceries".into(),
                        values: vec!["expenses:groceries".into(), "food".into()],
                    },
                    TableRow {
                        pattern: "salary".into(),
                        values: vec!["income:salary".into(), "".into()],
                    },
                ],
            }],
            ..Default::default()
        };
        let text = generate(&config);
        assert!(text.contains("if,account2,comment\n"));
        assert!(text.contains("groceries,expenses:groceries,food\n"));
        assert!(text.contains("salary,income:salary,\n"));
    }

    #[test]
    fn invalid_regex_is_rejected() {
        let config = RulesConfig {
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![Matcher {
                        pattern: "[invalid".into(),
                        field: None,
                        negate: false,
                    }],
                }],
                assignments: vec![],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };
        assert!(generate_rules_text(&config, &[]).is_err());
    }

    #[test]
    fn include_directive() {
        let config = RulesConfig {
            includes: vec![42],
            ..Default::default()
        };
        let paths = vec![(42i64, "/data/files/1/abc_other.rules".to_string())];
        let text = generate_rules_text(&config, &paths).expect("ok");
        assert!(text.contains("include /data/files/1/abc_other.rules\n"));
    }

    #[test]
    fn full_config_generation() {
        let config = RulesConfig {
            skip: Some(1),
            separator: Some(Separator::Comma),
            date_format: Some("%d/%m/%Y".into()),
            fields: vec![
                "date".into(),
                "description".into(),
                "".into(),
                "amount".into(),
            ],
            assignments: vec![FieldAssignment {
                field: "account1".into(),
                value: "assets:bank:checking".into(),
            }],
            conditionals: vec![ConditionalRule {
                rule_type: ConditionalType::Block,
                match_groups: vec![MatchGroup {
                    matchers: vec![Matcher {
                        pattern: "SUPERMARKET".into(),
                        field: None,
                        negate: false,
                    }],
                }],
                assignments: vec![FieldAssignment {
                    field: "account2".into(),
                    value: "expenses:groceries".into(),
                }],
                skip: false,
                end: false,
                table_fields: vec![],
                table_rows: vec![],
            }],
            ..Default::default()
        };

        let text = generate(&config);
        assert!(text.contains("skip 1\n"));
        assert!(text.contains("separator comma\n"));
        assert!(text.contains("date-format %d/%m/%Y\n"));
        assert!(text.contains("fields date, description, , amount\n"));
        assert!(text.contains("account1 assets:bank:checking\n"));
        assert!(text.contains("if SUPERMARKET\n"));
        assert!(text.contains("  account2 expenses:groceries\n"));
    }
}
