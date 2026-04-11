import { useState, useRef, useEffect } from 'react';
import { DayPicker } from 'react-day-picker';
import 'react-day-picker/style.css';
import { toISODate, startOfMonth, monthsAgo } from '../lib/format';
import './DateRangePicker.css';

interface DateRangePickerProps {
  begin?: string;
  end?: string;
  onChange: (begin: string, end: string) => void;
}

const today = () => new Date();

const PRESETS = [
  {
    label: 'This month',
    getRange: () => ({ begin: toISODate(startOfMonth(today())), end: toISODate(today()) }),
  },
  {
    label: 'Last month',
    getRange: () => {
      const d = today();
      const start = new Date(d.getFullYear(), d.getMonth() - 1, 1);
      const end = new Date(d.getFullYear(), d.getMonth(), 0);
      return { begin: toISODate(start), end: toISODate(end) };
    },
  },
  {
    label: 'This quarter',
    getRange: () => {
      const d = today();
      const q = Math.floor(d.getMonth() / 3);
      const start = new Date(d.getFullYear(), q * 3, 1);
      return { begin: toISODate(start), end: toISODate(d) };
    },
  },
  {
    label: 'This year',
    getRange: () => {
      const d = today();
      return { begin: `${d.getFullYear()}-01-01`, end: toISODate(d) };
    },
  },
  {
    label: 'Last 12 months',
    getRange: () => ({ begin: toISODate(monthsAgo(12, today())), end: toISODate(today()) }),
  },
  {
    label: 'All time',
    getRange: () => ({ begin: '', end: '' }),
  },
];

export function DateRangePicker({ begin, end, onChange }: DateRangePickerProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  const label = begin && end
    ? `${begin} → ${end}`
    : begin
    ? `From ${begin}`
    : end
    ? `Until ${end}`
    : 'All time';

  const selected = begin && end
    ? { from: new Date(begin + 'T00:00:00'), to: new Date(end + 'T00:00:00') }
    : undefined;

  return (
    <div className="drp" ref={ref}>
      <button className="drp-trigger" onClick={() => setOpen(v => !v)} type="button">
        <span className="drp-trigger-icon">📅</span>
        {label}
        <span className="drp-trigger-caret">▾</span>
      </button>

      {open && (
        <div className="drp-popover">
          <div className="drp-presets">
            {PRESETS.map(p => (
              <button
                key={p.label}
                className="drp-preset"
                type="button"
                onClick={() => {
                  const { begin: b, end: e } = p.getRange();
                  onChange(b, e);
                  setOpen(false);
                }}
              >
                {p.label}
              </button>
            ))}
          </div>
          <div className="drp-calendar">
            <DayPicker
              mode="range"
              selected={selected}
              onSelect={range => {
                if (range?.from && range?.to) {
                  onChange(toISODate(range.from), toISODate(range.to));
                  setOpen(false);
                } else if (range?.from) {
                  onChange(toISODate(range.from), '');
                }
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
