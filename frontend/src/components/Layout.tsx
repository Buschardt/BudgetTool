import { Outlet, NavLink } from 'react-router-dom';
import { useAuth } from '../auth';
import { ErrorBoundary } from './ErrorBoundary';
import './Layout.css';

const NAV_LINKS = [
  { to: '/', label: 'Dashboard', end: true },
  { to: '/balance', label: 'Balance', end: false },
  { to: '/income', label: 'Income', end: false },
  { to: '/register', label: 'Register', end: false },
  { to: '/cashflow', label: 'Cash Flow', end: false },
  { to: '/files', label: 'Files', end: false },
];

export function Layout() {
  const { username, logout } = useAuth();

  return (
    <div className="app">
      <nav className="nav">
        <span className="nav-brand">BudgetTool</span>
        <div className="nav-user">
          <span className="nav-username">{username}</span>
          <button onClick={logout} className="nav-logout">
            Log out
          </button>
        </div>
      </nav>
      <div className="tab-bar">
        {NAV_LINKS.map(link => (
          <NavLink
            key={link.to}
            to={link.to}
            end={link.end}
            className={({ isActive }) => `tab-link${isActive ? ' tab-link--active' : ''}`}
          >
            {link.label}
          </NavLink>
        ))}
      </div>
      <main className="main">
        <ErrorBoundary>
          <Outlet />
        </ErrorBoundary>
      </main>
    </div>
  );
}
