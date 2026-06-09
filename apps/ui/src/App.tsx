import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import RolesPage from "./pages/RolesPage";
import AccessKeysPage from "./pages/AccessKeysPage";
import SnapshotPage from "./pages/SnapshotPage";
import OtpInspectorPage from "./pages/OtpInspectorPage";
import UsersPage, { UsersListTab } from "./pages/UsersPage";
import { CustomAttributesTab } from "./components/CustomAttributesTab";
import AuthMethodsPage from "./pages/AuthMethodsPage";
import TenantsPage from "./pages/TenantsPage";
import IdentityProvidersPage from "./pages/IdentityProvidersPage";
import ResetPage from "./pages/ResetPage";
import { Layout } from "./components/Layout";
import { PageHeader } from "./components/PageHeader";
import { ConnectionOverlay } from "./components/ConnectionOverlay";
import NotFoundPage from "./pages/NotFoundPage";

function Placeholder({
  title,
  description,
}: {
  title: string;
  description?: string;
}) {
  return (
    <Layout title={title}>
      <PageHeader title={title} description={description} />
      <div className="empty-state">
        <div className="empty-state-icon">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        </div>
        <div className="empty-state-title">Coming soon</div>
        <div className="empty-state-description">
          This page is under construction.
        </div>
      </div>
    </Layout>
  );
}

const DASHBOARD_CARDS = [
  {
    href: "/auth-methods",
    title: "Auth Methods",
    desc: "Configure OTP, magic link, OAuth, SSO, and more",
    icon: (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="5" y="11" width="14" height="10" rx="2" />
        <path d="M8 11V7a4 4 0 0 1 8 0v4" />
      </svg>
    ),
  },
  {
    href: "/users",
    title: "Users",
    desc: "Create and manage test users for your sandbox",
    icon: (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
    ),
  },
  {
    href: "/roles",
    title: "Roles & Permissions",
    desc: "Set up RBAC for your local project",
    icon: (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
      </svg>
    ),
  },
  {
    href: "/tenants",
    title: "Tenants",
    desc: "Manage tenant organizations and SSO config",
    icon: (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="2" y="7" width="20" height="14" rx="2" />
        <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" />
      </svg>
    ),
  },
  {
    href: "/snapshot",
    title: "Snapshot",
    desc: "Export and import full emulator state as JSON",
    icon: (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <polyline points="21 8 21 21 3 21 3 8" />
        <rect x="1" y="3" width="22" height="5" />
        <line x1="10" y1="12" x2="14" y2="12" />
      </svg>
    ),
  },
  {
    href: "/otp-inspector",
    title: "OTP Inspector",
    desc: "View pending OTP codes in real-time",
    icon: (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
    ),
  },
];

function Dashboard() {
  return (
    <Layout title="Dashboard">
      <PageHeader
        title="Rescope"
        description="Local auth sandbox — isolated from Descope cloud"
      />
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: "var(--space-4)",
        }}
      >
        {DASHBOARD_CARDS.map((card) => (
          <a key={card.href} href={card.href} className="feature-card">
            <div className="feature-card-icon">{card.icon}</div>
            <div className="feature-card-title">{card.title}</div>
            <div className="feature-card-desc">{card.desc}</div>
          </a>
        ))}
      </div>
    </Layout>
  );
}

export default function App() {
  return (
    <>
      <ConnectionOverlay />
      <BrowserRouter basename={import.meta.env.BASE_URL}>
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<Dashboard />} />

          {/* Authentication */}
          <Route path="/auth-methods" element={<AuthMethodsPage />} />
          <Route path="/users" element={<UsersPage />}>
            <Route index element={<UsersListTab />} />
            <Route path="attributes" element={<CustomAttributesTab />} />
          </Route>
          <Route path="/access-keys" element={<AccessKeysPage />} />

          {/* Authorization */}
          <Route path="/roles" element={<RolesPage />} />
          <Route path="/tenants" element={<TenantsPage />} />
          <Route path="/identity-providers" element={<IdentityProvidersPage />} />

          {/* Settings */}
          <Route
            path="/settings/jwt-templates"
            element={
              <Placeholder
                title="JWT Templates"
                description="Customize the JWT claims issued to users"
              />
            }
          />
          <Route
            path="/connectors"
            element={
              <Placeholder
                title="Connectors"
                description="Integrate with external services"
              />
            }
          />

          {/* Emulator tools */}
          <Route path="/snapshot" element={<SnapshotPage />} />
          <Route path="/otp-inspector" element={<OtpInspectorPage />} />
          <Route path="/reset" element={<ResetPage />} />

          {/* Fallback */}
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </BrowserRouter>
    </>
  );
}
