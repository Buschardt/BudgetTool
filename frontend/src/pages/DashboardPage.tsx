import { useAuth } from '../auth';

export function DashboardPage() {
  const { username } = useAuth();

  return (
    <div>
      <h1>Welcome, {username}</h1>
      <p style={{ color: '#aaa', marginTop: '0.5rem' }}>
        Dashboard coming in Phase 6.
      </p>
    </div>
  );
}
