import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AuthProvider } from './auth';
import { ProtectedRoute } from './components/ProtectedRoute';
import { Layout } from './components/Layout';
import { LoginPage } from './pages/LoginPage';
import { DashboardPage } from './pages/DashboardPage';
import { BalancePage } from './pages/BalancePage';
import { IncomePage } from './pages/IncomePage';
import { RegisterPage } from './pages/RegisterPage';
import { CashflowPage } from './pages/CashflowPage';
import { FilesPage } from './pages/FilesPage';
import { RulesPage } from './pages/RulesPage';
import { RulesEditorPage } from './pages/RulesEditorPage';
import './index.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route element={<ProtectedRoute />}>
            <Route element={<Layout />}>
              <Route index element={<DashboardPage />} />
              <Route path="/balance" element={<BalancePage />} />
              <Route path="/income" element={<IncomePage />} />
              <Route path="/register" element={<RegisterPage />} />
              <Route path="/cashflow" element={<CashflowPage />} />
              <Route path="/files" element={<FilesPage />} />
              <Route path="/rules" element={<RulesPage />} />
              <Route path="/rules/new" element={<RulesEditorPage />} />
              <Route path="/rules/:id" element={<RulesEditorPage />} />
            </Route>
          </Route>
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  </StrictMode>
);
