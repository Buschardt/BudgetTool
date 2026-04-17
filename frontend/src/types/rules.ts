// TypeScript interfaces for hledger rules configurations.
// These mirror the Rust structs in api/src/rules_gen.rs.

export interface Matcher {
  pattern: string;
  field?: string; // if set, matches only this CSV field (e.g. "description")
  negate?: boolean;
}

export interface MatchGroup {
  matchers: Matcher[]; // AND-combined within the group
}

export interface FieldAssignment {
  field: string; // hledger field name: account1, description, comment, etc.
  value: string; // value with %interpolation: "expenses:food", "%2 - %3"
}

export interface TableRow {
  pattern: string;
  values: string[];
}

export interface ConditionalRule {
  type: 'block' | 'table';
  // Block-type fields
  matchGroups?: MatchGroup[]; // OR-combined between groups
  assignments?: FieldAssignment[];
  skip?: boolean;
  end?: boolean;
  // Table-type fields
  tableFields?: string[];
  tableRows?: TableRow[];
}

export interface RulesConfig {
  skip?: number;
  separator?: 'comma' | 'semicolon' | 'tab' | 'space';
  dateFormat?: string;
  decimalMark?: '.' | ',';
  newestFirst?: boolean;
  intraDayReversed?: boolean;
  balanceType?: '=' | '=*' | '==' | '==*';
  encoding?: string;
  timezone?: string;

  fields?: string[]; // names for each CSV column in order; "" = unnamed
  assignments?: FieldAssignment[];
  conditionals?: ConditionalRule[];
  includes?: number[]; // IDs of other rules configs to include
}

export const EMPTY_RULES_CONFIG: RulesConfig = {
  fields: [],
  assignments: [],
  conditionals: [],
  includes: [],
};

// API response types
export interface RulesConfigSummary {
  id: number;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface RulesConfigDetail extends RulesConfigSummary {
  config: RulesConfig;
}

// Valid hledger field names (for dropdowns)
export const HLEDGER_FIELDS = [
  'date',
  'date2',
  'status',
  'code',
  'description',
  'comment',
  'account1',
  'account2',
  'account3',
  'account4',
  'account5',
  'account6',
  'account7',
  'account8',
  'account9',
  'account10',
  'amount',
  'amount-in',
  'amount-out',
  'amount1',
  'amount2',
  'amount3',
  'amount1-in',
  'amount1-out',
  'amount2-in',
  'amount2-out',
  'amount3-in',
  'amount3-out',
  'currency',
  'currency1',
  'currency2',
  'currency3',
  'balance',
  'balance1',
  'balance2',
  'balance3',
  'comment1',
  'comment2',
  'comment3',
] as const;
