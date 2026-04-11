import type {
  AccountBalance,
  HledgerAmount,
  HledgerMixedAmount,
  HledgerQuantity,
  IncomeExpenseSummary,
  RegisterEntry,
} from '../types/hledger';

// ---------------------------------------------------------------------------
// Low-level amount helpers
// ---------------------------------------------------------------------------

function parseQuantity(q: HledgerQuantity): number {
  return q.decimalMantissa / Math.pow(10, q.decimalPlaces);
}

export function parseAmount(amt: HledgerAmount): number {
  return parseQuantity(amt.aquantity);
}

function primaryAmount(mixed: HledgerMixedAmount): { amount: number; commodity: string } {
  const amounts = mixed?.contents ?? [];
  if (amounts.length === 0) return { amount: 0, commodity: '$' };
  // Prefer the first non-zero amount, otherwise use the first
  const nonZero = amounts.find(a => parseAmount(a) !== 0) ?? amounts[0];
  return {
    amount: parseAmount(nonZero),
    commodity: nonZero.acommodity || '$',
  };
}

// ---------------------------------------------------------------------------
// balance / cashflow
// ---------------------------------------------------------------------------

// hledger balance --output-format=json returns something like:
// [["account:name", [mixedAmount], [mixedAmount]], ..., ["totals", [mixedAmount], [mixedAmount]]]
// or a richer object depending on flags. We handle both.

function parseBalanceRow(row: unknown): AccountBalance | null {
  if (!Array.isArray(row) || row.length < 2) return null;
  const name = typeof row[0] === 'string' ? row[0] : String(row[0]);
  if (name === 'totals') return null; // skip totals row
  const rawAmounts = row[1];
  // mixedAmount may be an array of HledgerAmount, or an object with .contents
  let amounts: HledgerAmount[] = [];
  if (Array.isArray(rawAmounts)) {
    amounts = rawAmounts as HledgerAmount[];
  } else if (rawAmounts && typeof rawAmounts === 'object' && 'contents' in rawAmounts) {
    amounts = (rawAmounts as HledgerMixedAmount).contents;
  }
  if (amounts.length === 0) return { account: name, amount: 0, commodity: '$' };
  const nonZero = amounts.find(a => parseAmount(a) !== 0) ?? amounts[0];
  return {
    account: name,
    amount: parseAmount(nonZero),
    commodity: nonZero.acommodity || '$',
  };
}

export function parseBalance(raw: unknown): AccountBalance[] {
  if (!Array.isArray(raw)) return [];
  return raw.flatMap(row => {
    const parsed = parseBalanceRow(row);
    return parsed ? [parsed] : [];
  });
}

export function parseCashflow(raw: unknown): AccountBalance[] {
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

    const amtRaw = r['amount'] as HledgerMixedAmount | undefined;
    const runningRaw = r['runningTotal'] as HledgerMixedAmount | undefined;

    const { amount, commodity } = amtRaw ? primaryAmount(amtRaw) : { amount: 0, commodity: '$' };
    const { amount: runningTotal } = runningRaw ? primaryAmount(runningRaw) : { amount: 0 };

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
