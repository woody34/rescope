import { api, type AuthMethodConfig } from "../lib/api";
import { Layout } from "../components/Layout";
import { PageHeader } from "../components/PageHeader";
import { useEffect, useState } from "react";

interface MethodRow {
  key: keyof AuthMethodConfig;
  label: string;
  description: string;
  icon: string;
}

const METHODS: MethodRow[] = [
  {
    key: "otp",
    label: "OTP (One-Time Password)",
    description: "Email and SMS one-time codes",
    icon: "📱",
  },
  {
    key: "magicLink",
    label: "Magic Link",
    description: "Passwordless email links",
    icon: "✉️",
  },
  {
    key: "password",
    label: "Password",
    description: "Traditional username + password",
    icon: "🔒",
  },
  {
    key: "totp",
    label: "TOTP (Authenticator App)",
    description: "Time-based one-time passwords via Google Authenticator etc.",
    icon: "🔐",
  },
  {
    key: "passkeys",
    label: "Passkeys / WebAuthn",
    description: "Biometric and hardware-key authentication",
    icon: "🗝️",
  },
  {
    key: "sso",
    label: "SSO / SAML",
    description: "Enterprise single sign-on via SAML providers",
    icon: "🏢",
  },
  {
    key: "enchantedLink",
    label: "Enchanted Link",
    description: "Magic link with cross-device verification",
    icon: "✨",
  },
  {
    key: "embeddedLink",
    label: "Embedded Link",
    description: "In-app deep-link authentication",
    icon: "🔗",
  },
  {
    key: "notp",
    label: "nOTP (Notification OTP)",
    description: "Push notification one-time codes",
    icon: "🔔",
  },
];

function isEnabled(
  config: AuthMethodConfig | null,
  key: keyof AuthMethodConfig,
): boolean {
  if (!config) return false;
  const method = config[key];
  if (typeof method === "object" && method !== null && "enabled" in method) {
    return (method as { enabled: boolean }).enabled;
  }
  return false;
}

export default function AuthMethodsPage() {
  const [config, setConfig] = useState<AuthMethodConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const { authMethods } = await api.authMethods.get();
        setConfig(authMethods);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const toggleMethod = async (key: keyof AuthMethodConfig) => {
    if (!config) return;
    setSaving(key as string);
    const method = config[key] as Record<string, unknown>;
    const newEnabled = !method["enabled"];
    const updated = {
      ...config,
      [key]: { ...method, enabled: newEnabled },
    };
    try {
      await api.authMethods.update(updated);
      setConfig(updated);
    } finally {
      setSaving(null);
    }
  };

  return (
    <Layout title="Auth Methods">
      <PageHeader
        title="Auth Methods"
        description="Configure which authentication methods are enabled for your project"
      />

      {loading ? (
        <div style={{ display: "flex", justifyContent: "center", padding: 40 }}>
          <div className="loading-spinner" />
        </div>
      ) : (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "var(--space-3)",
          }}
        >
          {METHODS.map(({ key, label, description, icon }) => {
            const enabled = isEnabled(config, key);
            const isSaving = saving === key;
            return (
              <div
                key={key}
                className="card"
                style={{
                  padding: "var(--space-4) var(--space-5)",
                  display: "flex",
                  alignItems: "center",
                  gap: "var(--space-4)",
                  transition: "opacity 0.15s",
                  opacity: isSaving ? 0.6 : 1,
                }}
              >
                <div style={{ fontSize: 28, flexShrink: 0 }}>{icon}</div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontWeight: "var(--fw-semibold)",
                      marginBottom: 2,
                    }}
                  >
                    {label}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--text-sm)",
                      color: "var(--color-text-secondary)",
                    }}
                  >
                    {description}
                  </div>
                </div>
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--space-3)",
                  }}
                >
                  <span
                    data-testid={`badge-${key}`}
                    className={`badge ${enabled ? "badge-success" : "badge-neutral"}`}
                  >
                    {enabled ? "Enabled" : "Disabled"}
                  </span>
                  <button
                    id={`toggle-${key}`}
                    data-testid={`toggle-${key}`}
                    className={`btn ${enabled ? "btn-secondary" : "btn-primary"} btn-sm`}
                    onClick={() => toggleMethod(key)}
                    disabled={isSaving}
                  >
                    {isSaving ? "Saving…" : enabled ? "Disable" : "Enable"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </Layout>
  );
}
