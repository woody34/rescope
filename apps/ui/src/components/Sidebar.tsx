import { NavLink, useLocation } from "react-router-dom";

// Inline SVG icon components (lucide-style, 15px stroke icons)
const Icons = {
  Dashboard: () => (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="3" y="3" width="7" height="7" rx="1" />
      <rect x="14" y="3" width="7" height="7" rx="1" />
      <rect x="14" y="14" width="7" height="7" rx="1" />
      <rect x="3" y="14" width="7" height="7" rx="1" />
    </svg>
  ),
  Lock: () => (
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
  Users: () => (
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
  Key: () => (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="7.5" cy="15.5" r="5.5" />
      <path d="M21 2l-9.6 9.6M15.5 7.5l3 3L22 7l-3-3" />
    </svg>
  ),
  Shield: () => (
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
  Building: () => (
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
  FileCode: () => (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <path d="M10 13l-2 2 2 2M14 13l2 2-2 2" />
    </svg>
  ),
  Plug: () => (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M12 22v-5M9 7V2M15 7V2M7 7h10l-1 8H8L7 7z" />
      <path d="M9 17c0 2.2 1.3 5 3 5s3-2.8 3-5" />
    </svg>
  ),
  Tag: () => (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
      <line x1="7" y1="7" x2="7.01" y2="7" />
    </svg>
  ),
  Archive: () => (
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
  Terminal: () => (
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
  RefreshCw: () => (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="23 4 23 10 17 10" />
      <polyline points="1 20 1 14 7 14" />
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
    </svg>
  ),
};

// Rescope logo SVG (scope reticle + R)
function RescopeLogo() {
  return (
    <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M4 10 L4 4 L10 4"
        stroke="#3fb950"
        strokeWidth="2.5"
        strokeLinecap="square"
      />
      <path
        d="M22 4 L28 4 L28 10"
        stroke="#3fb950"
        strokeWidth="2.5"
        strokeLinecap="square"
      />
      <path
        d="M4 22 L4 28 L10 28"
        stroke="#3fb950"
        strokeWidth="2.5"
        strokeLinecap="square"
      />
      <path
        d="M22 28 L28 28 L28 22"
        stroke="#3fb950"
        strokeWidth="2.5"
        strokeLinecap="square"
      />
      <circle cx="16" cy="6" r="1" fill="#3fb950" opacity="0.4" />
      <circle cx="16" cy="26" r="1" fill="#3fb950" opacity="0.4" />
      <circle cx="6" cy="16" r="1" fill="#3fb950" opacity="0.4" />
      <circle cx="26" cy="16" r="1" fill="#3fb950" opacity="0.4" />
      <path
        d="M12 10 L12 22"
        stroke="#e6edf3"
        strokeWidth="2"
        strokeLinecap="round"
      />
      <path
        d="M12 10 L17 10 Q20 10 20 13 Q20 16 17 16 L12 16"
        stroke="#e6edf3"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M17 16 L21 22"
        stroke="#e6edf3"
        strokeWidth="2"
        strokeLinecap="round"
      />
    </svg>
  );
}

const NAV = [
  {
    section: "Project",
    items: [{ to: "/dashboard", icon: Icons.Dashboard, label: "Dashboard" }],
  },
  {
    section: "Authentication",
    items: [
      { to: "/auth-methods", icon: Icons.Lock, label: "Auth Methods" },
      { to: "/users", icon: Icons.Users, label: "Users" },
      { to: "/access-keys", icon: Icons.Key, label: "Access Keys" },
    ],
  },
  {
    section: "Authorization",
    items: [
      { to: "/roles", icon: Icons.Shield, label: "Roles & Permissions" },
      { to: "/tenants", icon: Icons.Building, label: "Tenants" },
      { to: "/identity-providers", icon: Icons.Key, label: "Identity Providers" },
    ],
  },
  {
    section: "Settings",
    items: [
      {
        to: "/settings/jwt-templates",
        icon: Icons.FileCode,
        label: "JWT Templates",
      },
      { to: "/connectors", icon: Icons.Plug, label: "Connectors" },
    ],
  },
  {
    section: "Emulator",
    items: [
      { to: "/snapshot", icon: Icons.Archive, label: "Snapshot" },
      { to: "/otp-inspector", icon: Icons.Terminal, label: "OTP Inspector" },
      { to: "/reset", icon: Icons.RefreshCw, label: "Reset" },
    ],
  },
];

export function Sidebar() {
  const location = useLocation();

  return (
    <nav className="sidebar" aria-label="Main navigation">
      <div className="sidebar-logo">
        <div className="sidebar-logo-icon">
          <RescopeLogo />
        </div>
        <div>
          <div className="sidebar-logo-text">Rescope</div>
          <div className="sidebar-logo-sub">local · sandboxed</div>
        </div>
      </div>

      <div className="sidebar-nav">
        {NAV.map((group) => (
          <div key={group.section}>
            <div className="sidebar-section-label">{group.section}</div>
            {group.items.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  `sidebar-link${isActive || location.pathname.startsWith(item.to) ? " active" : ""}`
                }
              >
                <span className="sidebar-link-icon">
                  <item.icon />
                </span>
                {item.label}
              </NavLink>
            ))}
          </div>
        ))}
      </div>

      <div className="sidebar-footer">
        <div className="sidebar-status-dot" />
        <span className="sidebar-status-label">emulator running</span>
      </div>
    </nav>
  );
}
