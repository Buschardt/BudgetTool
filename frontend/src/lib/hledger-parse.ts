import type {
  AccountBalance,
  IncomeExpenseSummary,
  RegisterEntry,
} from '../types/hledger';

// ---------------------------------------------------------------------------
// Low-level amount helpers
// ---------------------------------------------------------------------------

function parseQuantity(q: unknown): number {
  if (q === null || q === undefined) return 0;
  if (typeof q === 'number') return q;
  if (typeof q === 'object') {
    const obj = q as Record<string, unknown>;
    const mantissa = typeof obj['decimalMantissa'] === 'number' ? obj['decimalMantissa'] : 0;
    const places = typeof obj['decimalPlaces'] === 'number' ? obj['decimalPlaces'] : 0;
    return mantissa / Math.pow(10, places);
  }
  return 0;
}

function safeParseAmount(amt: unknown): number {
  if (!amt || typeof amt !== 'object') return 0;
  const a = amt as Record<string, unknown>;
  return parseQuantity(a['aquantity']);
}

function safeCommodity(amt: unknown): string {
  if (!amt || typeof amt !== 'object') return '$';
  const a = amt as Record<string, unknown>;
  return typeof a['acommodity'] === 'string' ? a['acommodity'] || '$' : '$';
}

/** Extract primary { amount, commodity } from a MixedAmount.
 *  Handles both array-form [Amount, ...] and object-form { contents: [Amount, ...] }. */
function primaryAmount(raw: unknown): { amount: number; commodity: string } {
  let amounts: unknown[] = [];
  if (Array.isArray(raw)) {
    amounts = raw;
  } else if (raw && typeof raw === 'object') {
    const obj = raw as Record<string, unknown>;
    if ('contents' in obj && Array.isArray(obj['contents'])) {
      amounts = obj['contents'];
    }
  }
  if (amounts.length === 0) return { amount: 0, commodity: '$' };
  const nonZero = amounts.find(a => safeParseAmount(a) !== 0) ?? amounts[0];
  return {
    amount: safeParseAmount(nonZero),
    commodity: safeCommodity(nonZero),
  };
}

// ---------------------------------------------------------------------------
// balance / cashflow
// ---------------------------------------------------------------------------

// hledger balance --output-format=json returns an array of 3-tuples:
//   [accountName, mixedAmount, cumulativeMixedAmount]
// The last row has an empty string "" as the account name (totals).
//
// With --period, it returns a compound report:
//   [[dateRange, rows, totals], ...]   (multi-period)
// We detect and flatten both.

function parseBalanceRow(row: unknown): AccountBalance | null {
  if (!Array.isArray(row) || row.length < 2) return null;

  // Detect multi-period structure: first element is a date-range array, not a string
  // e.g. [["2026-01-01", "2026-02-01"], rows, totals]
  if (Array.isArray(row[0])) return null;

  const name = typeof row[0] === 'string' ? row[0] : String(row[0]);
  // Skip totals rows (empty string or "totals")
  if (name === '' || name === 'totals') return null;

  const { amount, commodity } = primaryAmount(row[1]);
  return { account: name, amount, commodity };
}

function isMultiPeriod(raw: unknown[]): boolean {
  // Multi-period: each element is [dateRange, rows, totals]
  // where dateRange is a 2-element array of date strings
  return raw.length > 0 && Array.isArray(raw[0]) && Array.isArray((raw[0] as unknown[])[0]);
}

export function parseBalance(raw: unknown): AccountBalance[] {
  if (!Array.isArray(raw) || raw.length === 0) return [];

  if (isMultiPeriod(raw)) {
    // Flatten all periods into a single list (last period = most recent)
    const lastPeriod = raw[raw.length - 1] as unknown[];
    const rows = lastPeriod[1];
    if (!Array.isArray(rows)) return [];
    return rows.flatMap(row => {
      const parsed = parseBalanceRow(row);
      return parsed ? [parsed] : [];
    });
  }

  return raw.flatMap(row => {
    const parsed = parseBalanceRow(row);
    return parsed ? [parsed] : [];
  });
}

// hledger cashflow --output-format=json returns a compound report like incomestatement:
// { title, subtitle, subreports: [{title, rows, totals}], totals }
export function parseCashflow(raw: unknown): AccountBalance[] {
  if (!raw || typeof raw !== 'object') return parseBalance(raw);
  const obj = raw as Record<string, unknown>;
  // Compound report with subreports
  if ('subreports' in obj && Array.isArray(obj['subreports'])) {
    return obj['subreports'].flatMap(section => parseSection(section));
  }
  // Fallback: try flat array format
  return parseBalance(raw);
}

// ---------------------------------------------------------------------------
// register
// ---------------------------------------------------------------------------

// hledger register --output-format=json returns an array of objects:
// { date, date2, description, account, amount: mixedAmount, runningTotal: mixedAmount }

export function parseRegister(raw: unknown): RegisterEntry[] {
  if (!Array.isArray(raw)) return [];
  return raw.flatMap(item => {
    if (!item || typeof item !== 'object') return [];
    const r = item as Record<string, unknown>;
    const date = typeof r['date'] === 'string' ? r['date'] : '';
    const description = typeof r['description'] === 'string' ? r['description'] : '';
    const account = typeof r['account'] === 'string' ? r['account'] : '';

    const { amount, commodity } = primaryAmount(r['amount'] as unknown);
    const { amount: runningTotal } = primaryAmount(r['runningTotal'] as unknown);

    return [{ date, description, account, amount, commodity, runningTotal }];
  });
}

// ---------------------------------------------------------------------------
// incomestatement
// ---------------------------------------------------------------------------

// hledger incomestatement --output-format=json returns:
// { title, subtitle, subreports: [ revenues_section, expenses_section ], totals }
// Each section: { title, rows: [[accountName, mixedAmount], ...], totals }

function parseSection(section: unknown): AccountBalance[] {
  if (!section || typeof section !== 'object') return [];
  const s = section as Record<string, unknown>;
  const rows = s['rows'];
  if (!Array.isArray(rows)) return [];
  return rows.flatMap(row => {
    const parsed = parseBalanceRow(row);
    return parsed ? [parsed] : [];
  });
}

export function parseIncomeStatement(raw: unknown): IncomeExpenseSummary {
  const empty: IncomeExpenseSummary = { revenues: [], expenses: [], netIncome: 0 };
  if (!raw || typeof raw !== 'object') return empty;

  const obj = raw as Record<string, unknown>;

  // subreports is an array: [revenues, expenses]
  const subreports = obj['subreports'];
  if (!Array.isArray(subreports) || subreports.length < 2) return empty;

  const revenues = parseSection(subreports[0]);
  const expenses = parseSection(subreports[1]);

  const totalRevenue = revenues.reduce((s, r) => s + r.amount, 0);
  const totalExpense = expenses.reduce((s, r) => s + Math.abs(r.amount), 0);
  const netIncome = totalRevenue - totalExpense;

  return { revenues, expenses, netIncome };
}
