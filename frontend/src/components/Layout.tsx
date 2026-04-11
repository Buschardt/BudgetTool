import { Outlet } from 'react-router-dom';
import { useAuth } from '../auth';
import './Layout.css';

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
      <main className="main">
        <Outlet />
      </main>
    </div>
  );
}
