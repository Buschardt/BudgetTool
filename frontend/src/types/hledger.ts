// Raw hledger --output-format=json types

export interface HledgerQuantity {
  decimalMantissa: number;
  decimalPlaces: number;
}

export interface HledgerAmount {
  acommodity: string;
  aquantity: HledgerQuantity;
  aprice: null | unknown;
  astyle: unknown;
}

export interface HledgerMixedAmount {
  contents: HledgerAmount[];
}

// balance / cashflow JSON: array of rows, each row is [accountName, amounts, cumulativeAmounts]
// The full structure is: { title, totalsrow: [labels, amounts, cumulativeAmounts] }
// or it can be a flat array depending on hledger version.
// We treat the raw output as unknown and parse defensively.

// register JSON: array of posting objects
export interface HledgerRegisterRow {
  date: string;
  date2: string | null;
  description: string;
  account: string;
  amount: HledgerMixedAmount;
  runningTotal: HledgerMixedAmount;
}

// incomestatement JSON
export interface HledgerIncomeStatementSection {
  title: string;
  rows: unknown[];
  totals: unknown;
}

export interface HledgerIncomeStatementRaw {
  title: string;
  subtitle: string;
  subreports: [HledgerIncomeStatementSection, HledgerIncomeStatementSection];
  totals: unknown;
}

// -----------------------------------------------
// App-level types (what components consume)
// -----------------------------------------------

export interface AccountBalance {
  account: string;
  amount: number;
  commodity: string;
}

export interface RegisterEntry {
  date: string;
  description: string;
  account: string;
  amount: number;
  commodity: string;
  runningTotal: number;
}

export interface IncomeExpenseSummary {
  revenues: AccountBalance[];
  expenses: AccountBalance[];
  netIncome: number;
}
