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
    // Prefer floatingPoint if available (hledger 1.33+)
    if (typeof obj['floatingPoint'] === 'number') return obj['floatingPoint'];
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
// balance
// ---------------------------------------------------------------------------

// hledger balance --output-format=json
//
// Old format (hledger < 1.33):
//   [[accountName, mixedAmount, cumulativeMixedAmount], ..., ["", totals]]
//
// New format (hledger 1.33+):
//   [[row, ...], totalMixedAmount]
//   where row = [fullAccountName, displayName, depth, [Amount, ...]]
//
// With --period, returns multi-period format (both old and new):
//   [[[dateStart, dateEnd], rows, totals], ...]

function parseOldBalanceRow(row: unknown): AccountBalance | null {
  if (!Array.isArray(row) || row.length < 2) return null;
  if (Array.isArray(row[0])) return null; // date-range row in multi-period
  const name = typeof row[0] === 'string' ? row[0] : String(row[0]);
  if (name === '' || name === 'totals') return null;
  const { amount, commodity } = primaryAmount(row[1]);
  return { account: name, amount, commodity };
}

function parseNewBalanceRow(row: unknown): AccountBalance | null {
  // New format: [fullName, displayName, depth, [amounts]]
  if (!Array.isArray(row) || row.length < 4) return null;
  const name = typeof row[0] === 'string' ? row[0] : null;
  if (!name || name === '' || name === 'totals') return null;
  const { amount, commodity } = primaryAmount(row[3]);
  return { account: name, amount, commodity };
}

function isNewBalanceFormat(raw: unknown[]): boolean {
  // New format: outer array has 2 elements [rowsArray, totalsMixedAmount]
  // and rowsArray[0] is a 4-element array [string, string, number, array]
  if (raw.length < 1 || !Array.isArray(raw[0])) return false;
  const rows = raw[0] as unknown[];
  if (rows.length === 0) return false;
  const firstRow = rows[0];
  if (!Array.isArray(firstRow) || (firstRow as unknown[]).length < 4) return false;
  const fr = firstRow as unknown[];
  return typeof fr[0] === 'string' && typeof fr[1] === 'string' && typeof fr[2] === 'number';
}

function isMultiPeriod(raw: unknown[]): boolean {
  // Multi-period: each element is [dateRange, rows, totals]
  // where dateRange is a 2-element array of date strings
  if (raw.length === 0 || !Array.isArray(raw[0])) return false;
  const first = raw[0] as unknown[];
  return Array.isArray(first[0]) && (first[0] as unknown[]).length === 2;
}

export function parseBalance(raw: unknown): AccountBalance[] {
  if (!Array.isArray(raw) || raw.length === 0) return [];

  if (isNewBalanceFormat(raw)) {
    // New format: raw[0] is the rows array
    return (raw[0] as unknown[]).flatMap(row => {
      const parsed = parseNewBalanceRow(row);
      return parsed ? [parsed] : [];
    });
  }

  if (isMultiPeriod(raw)) {
    // Flatten all periods, use last period (most recent)
    const lastPeriod = raw[raw.length - 1] as unknown[];
    const rows = lastPeriod[1];
    if (!Array.isArray(rows)) return [];
    return rows.flatMap(row => {
      const parsed = parseOldBalanceRow(row);
      return parsed ? [parsed] : [];
    });
  }

  // Old flat format
  return raw.flatMap(row => {
    const parsed = parseOldBalanceRow(row);
    return parsed ? [parsed] : [];
  });
}

// ---------------------------------------------------------------------------
// incomestatement / cashflow
// ---------------------------------------------------------------------------

// Old format:
//   { title, subreports: [{ title, rows: [[accountName, amounts], ...], totals }], totals }
//
// New format (hledger 1.33+):
//   { cbrTitle, cbrSubreports: [[title, { prRows: [{ prrName, prrAmounts, ... }], prDates }, totals], ...] }

function parseNewReportRow(row: unknown): AccountBalance | null {
  if (!row || typeof row !== 'object') return null;
  const r = row as Record<string, unknown>;
  const name = typeof r['prrName'] === 'string' ? r['prrName'] : null;
  if (!name || name === '' || name === 'totals') return null;

  // prrAmounts: [[Amount, ...], ...] — one sub-array per column/period
  const prrAmounts = r['prrAmounts'];
  if (!Array.isArray(prrAmounts) || prrAmounts.length === 0) {
    return { account: name, amount: 0, commodity: '$' };
  }
  // Use first period's amounts
  const { amount, commodity } = primaryAmount(prrAmounts[0]);
  return { account: name, amount, commodity };
}

function parseNewSubreport(sr: unknown): AccountBalance[] {
  // New subreport: [title, { prRows: [...] }, totals]
  if (!Array.isArray(sr) || sr.length < 2) return [];
  const report = sr[1];
  if (!report || typeof report !== 'object') return [];
  const r = report as Record<string, unknown>;
  const rows = r['prRows'];
  if (!Array.isArray(rows)) return [];
  return rows.flatMap(row => {
    const parsed = parseNewReportRow(row);
    return parsed ? [parsed] : [];
  });
}

function parseOldSection(section: unknown): AccountBalance[] {
  if (!section || typeof section !== 'object') return [];
  const s = section as Record<string, unknown>;
  const rows = s['rows'];
  if (!Array.isArray(rows)) return [];
  return rows.flatMap(row => {
    const parsed = parseOldBalanceRow(row);
    return parsed ? [parsed] : [];
  });
}

export function parseCashflow(raw: unknown): AccountBalance[] {
  if (!raw || typeof raw !== 'object') return parseBalance(raw);
  const obj = raw as Record<string, unknown>;

  // New format
  if ('cbrSubreports' in obj && Array.isArray(obj['cbrSubreports'])) {
    return (obj['cbrSubreports'] as unknown[]).flatMap(sr => parseNewSubreport(sr));
  }

  // Old format
  if ('subreports' in obj && Array.isArray(obj['subreports'])) {
    return (obj['subreports'] as unknown[]).flatMap(sr => parseOldSection(sr));
  }

  return parseBalance(raw);
}

export function parseIncomeStatement(raw: unknown): IncomeExpenseSummary {
  const empty: IncomeExpenseSummary = { revenues: [], expenses: [], netIncome: 0 };
  if (!raw || typeof raw !== 'object') return empty;
  const obj = raw as Record<string, unknown>;

  // New format: cbrSubreports = [[title, {prRows}, totals], ...]
  if ('cbrSubreports' in obj && Array.isArray(obj['cbrSubreports'])) {
    const srs = obj['cbrSubreports'] as unknown[];
    if (srs.length < 2) return empty;
    const revenues = parseNewSubreport(srs[0]);
    const expenses = parseNewSubreport(srs[1]);
    const totalRevenue = revenues.reduce((s, r) => s + r.amount, 0);
    const totalExpense = expenses.reduce((s, r) => s + Math.abs(r.amount), 0);
    return { revenues, expenses, netIncome: totalRevenue - totalExpense };
  }

  // Old format: subreports = [{title, rows}, ...]
  if ('subreports' in obj && Array.isArray(obj['subreports'])) {
    const subreports = obj['subreports'] as unknown[];
    if (subreports.length < 2) return empty;
    const revenues = parseOldSection(subreports[0]);
    const expenses = parseOldSection(subreports[1]);
    const totalRevenue = revenues.reduce((s, r) => s + r.amount, 0);
    const totalExpense = expenses.reduce((s, r) => s + Math.abs(r.amount), 0);
    return { revenues, expenses, netIncome: totalRevenue - totalExpense };
  }

  return empty;
}

// ---------------------------------------------------------------------------
// register
// ---------------------------------------------------------------------------

// hledger register --output-format=json returns an array of 5-tuples:
// [date|null, date2|null, description|null, posting, runningTotalMixedAmount]
// date and description are null on continuation rows (non-first postings of a
// multi-posting transaction); we carry forward the last non-null values.
// posting.paccount holds the account name; posting.pamount is a MixedAmount array.

export function parseRegister(raw: unknown): RegisterEntry[] {
  if (!Array.isArray(raw)) return [];
  const out: RegisterEntry[] = [];
  let lastDate = '';
  let lastDescription = '';
  for (const item of raw) {
    if (!Array.isArray(item) || item.length < 5) continue;
    const date = typeof item[0] === 'string' ? item[0] : lastDate;
    const description = typeof item[2] === 'string' ? item[2] : lastDescription;
    lastDate = date;
    lastDescription = description;

    const posting = item[3];
    let account = '';
    let pamount: unknown = null;
    if (posting && typeof posting === 'object') {
      const p = posting as Record<string, unknown>;
      if (typeof p['paccount'] === 'string') account = p['paccount'];
      pamount = p['pamount'];
    }
    const { amount, commodity } = primaryAmount(pamount);
    const { amount: runningTotal } = primaryAmount(item[4]);

    out.push({ date, description, account, amount, commodity, runningTotal });
  }
  return out;
}
