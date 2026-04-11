export function formatCurrency(amount: number, commodity = '$'): string {
  const abs = Math.abs(amount);
  const formatted = abs.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  const sign = amount < 0 ? '-' : '';
  // If commodity looks like a symbol (short), prefix it; otherwise suffix
  if (commodity.length <= 3 && /^[^a-z]/i.test(commodity)) {
    return `${sign}${commodity}${formatted}`;
  }
  return `${sign}${formatted} ${commodity}`;
}

export function formatDate(iso: string): string {
  if (!iso) return '';
  // hledger dates are YYYY-MM-DD
  const [year, month, day] = iso.split('-');
  if (!year || !month || !day) return iso;
  const d = new Date(Number(year), Number(month) - 1, Number(day));
  return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** Return ISO YYYY-MM-DD string for a Date object */
export function toISODate(d: Date): string {
  return d.toISOString().slice(0, 10);
}

/** First day of the current month */
export function startOfMonth(d = new Date()): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1);
}

/** First day N months ago */
export function monthsAgo(n: number, from = new Date()): Date {
  return new Date(from.getFullYear(), from.getMonth() - n, 1);
}
