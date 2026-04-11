import './SummaryCard.css';

interface SummaryCardProps {
  title: string;
  value: string;
  subtitle?: string;
  trend?: 'up' | 'down' | 'neutral';
}

export function SummaryCard({ title, value, subtitle, trend }: SummaryCardProps) {
  return (
    <div className="summary-card">
      <div className="summary-card-title">{title}</div>
      <div className={`summary-card-value ${trend ? `summary-card-value--${trend}` : ''}`}>
        {value}
      </div>
      {subtitle && <div className="summary-card-subtitle">{subtitle}</div>}
    </div>
  );
}
