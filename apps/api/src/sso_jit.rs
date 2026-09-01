//! Just-in-time user provisioning for SSO sign-ins.
//!
//! Real Descope provisions a user the first time an identity provider asserts
//! them, so a working SSO tenant lets *any* address on its domains sign in. The
//! emulator has no upstream IdP to read attributes from, so it derives what it
//! can from the email address, applies the tenant's defaults, and — when the
//! tenant asks for it — collects the rest from the person signing in via the
//! configuration screen in [`crate::routes::emulator::sso_provision`].
//!
//! Only SSO surfaces call this. Password, OTP, magic-link, and flow sign-ins
//! keep rejecting unknown login IDs.

use std::collections::HashMap;

use serde_json::Value;

use crate::{
    error::EmulatorError,
    state::EmulatorState,
    store::user_store::new_user_id,
    types::{Tenant, User, UserTenant},
};

/// Attributes an SSO surface can supply for the user it is provisioning.
/// Anything left empty falls back to the tenant defaults, then to values
/// derived from the email address.
#[derive(Debug, Default, Clone)]
pub struct SsoProvisionInput {
    pub email: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub role_names: Vec<String>,
    pub custom_attributes: HashMap<String, Value>,
}

impl SsoProvisionInput {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            ..Default::default()
        }
    }
}

/// Normalize an email-shaped login ID the way user creation does: lowercase,
/// trimmed. Returns `None` when the value is not an email address.
pub fn normalize_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') || !domain.contains('.') {
        return None;
    }
    Some(email)
}

/// Turn an email local-part into a readable display name: `jane.doe` → `Jane
/// Doe`. Purely cosmetic — it gives provisioned users a name in pickers and
/// tables instead of a bare address.
pub fn display_name_from_email(email: &str) -> String {
    let local = email.split('@').next().unwrap_or(email);
    local
        .split(['.', '_', '-', '+'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The SSO tenant that owns an email address's domain.
pub async fn resolve_sso_tenant(
    state: &EmulatorState,
    email: &str,
) -> Result<Tenant, EmulatorError> {
    let tenants = state.tenants.read().await;
    tenants.find_by_email(email).cloned()
}

/// Create the user for an SSO sign-in, or return the existing one.
///
/// Idempotent on purpose: several surfaces (the SAML assertion consumer, the
/// IdP pickers, the configuration screen) can race on the same address within a
/// single sign-in, and a duplicate must not break the login.
pub async fn provision_sso_user(
    state: &EmulatorState,
    tenant: &Tenant,
    input: SsoProvisionInput,
) -> Result<User, EmulatorError> {
    let email = normalize_email(&input.email).ok_or_else(|| {
        EmulatorError::ValidationError(format!("`{}` is not an email address", input.email))
    })?;

    // A tenant that declares SSO domains only vouches for addresses on them.
    // A tenant with no domains is reached only by naming it explicitly — an
    // identity provider asserting into its own tenant — so there is nothing to
    // check the address against.
    if tenant.all_domains().next().is_some() && !tenant.owns_email(&email) {
        return Err(EmulatorError::ValidationError(format!(
            "`{email}` is not on a domain owned by tenant `{}`",
            tenant.id
        )));
    }

    let jit = tenant.sso_jit();
    if !jit.enabled {
        return Err(EmulatorError::UserNotFound);
    }

    // Existing user wins — provisioning never overwrites configuration someone
    // seeded or edited deliberately.
    if let Ok(existing) = state.users.read().await.load(&email) {
        return Ok(existing.clone());
    }

    let role_names = if !input.role_names.is_empty() {
        input.role_names
    } else if !jit.default_role_names.is_empty() {
        jit.default_role_names.clone()
    } else {
        state.roles.read().await.default_role_names()
    };

    let mut custom_attributes = jit.default_custom_attributes.clone();
    custom_attributes.extend(input.custom_attributes);

    let name = input
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| display_name_from_email(&email));

    let user = User {
        user_id: new_user_id(),
        login_ids: vec![email.clone()],
        email: Some(email.clone()),
        name: Some(name),
        given_name: input.given_name.filter(|v| !v.trim().is_empty()),
        family_name: input.family_name.filter(|v| !v.trim().is_empty()),
        // The IdP vouched for the address, so it arrives verified — matching a
        // real assertion, and letting the user straight through email gates.
        verified_email: true,
        role_names: role_names.clone(),
        user_tenants: vec![UserTenant {
            tenant_id: tenant.id.clone(),
            tenant_name: tenant.name.clone(),
            role_names,
        }],
        // Provisioned users are signing in right now, so they are active
        // immediately rather than "invited" like a management-API creation.
        status: "enabled".into(),
        created_time: now_secs(),
        saml: true,
        custom_attributes,
        ..Default::default()
    };

    match state.users.write().await.insert(user) {
        Ok(()) => {}
        // Lost a race against a concurrent sign-in for the same address.
        Err(EmulatorError::UserAlreadyExists) => {}
        Err(e) => return Err(e),
    }

    let users = state.users.read().await;
    Ok(users.load(&email)?.clone())
}

/// Provision into a tenant named by its ID — the shape the identity-provider
/// emulator endpoints use, where the IdP already determines the tenant.
pub async fn provision_sso_user_for_tenant(
    state: &EmulatorState,
    tenant_id: &str,
    input: SsoProvisionInput,
) -> Result<User, EmulatorError> {
    let tenant = state.tenants.read().await.load(tenant_id)?.clone();
    provision_sso_user(state, &tenant, input).await
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::EmulatorConfig, types::AuthType};

    async fn make_state() -> EmulatorState {
        EmulatorState::new(&EmulatorConfig::default())
            .await
            .unwrap()
    }

    fn sso_tenant() -> Tenant {
        Tenant {
            id: "acme".into(),
            name: "Acme".into(),
            domains: vec!["acme.test".into()],
            auth_type: AuthType::Saml,
            ..Default::default()
        }
    }

    /// Why: the whole point of JIT provisioning is that an address nobody seeded
    ///      can sign in, and the resulting user has to be usable — active, email
    ///      verified, and a member of the tenant that vouched for it.
    /// Decision: assert the full provisioned shape, not just that a user exists.
    #[tokio::test]
    async fn provisions_an_active_verified_tenant_member() {
        let state = make_state().await;
        let tenant = sso_tenant();

        let user = provision_sso_user(
            &state,
            &tenant,
            SsoProvisionInput::new("Jane.Doe@Acme.test"),
        )
        .await
        .unwrap();

        assert_eq!(user.login_ids, vec!["jane.doe@acme.test"]);
        assert_eq!(user.email.as_deref(), Some("jane.doe@acme.test"));
        assert_eq!(user.name.as_deref(), Some("Jane Doe"));
        assert!(user.verified_email);
        assert!(user.saml);
        assert_eq!(user.status, "enabled");
        assert_eq!(user.user_tenants.len(), 1);
        assert_eq!(user.user_tenants[0].tenant_id, "acme");
        assert_eq!(user.user_tenants[0].tenant_name, "Acme");
    }

    /// Why: a sign-in can reach provisioning more than once (picker, assertion,
    ///      config screen), and the second pass must not fail the login or
    ///      clobber a user someone configured deliberately.
    /// Decision: return the stored user untouched.
    #[tokio::test]
    async fn provisioning_an_existing_user_returns_it_unchanged() {
        let state = make_state().await;
        let tenant = sso_tenant();

        let first = provision_sso_user(
            &state,
            &tenant,
            SsoProvisionInput {
                name: Some("Original Name".into()),
                ..SsoProvisionInput::new("dup@acme.test")
            },
        )
        .await
        .unwrap();

        let second = provision_sso_user(
            &state,
            &tenant,
            SsoProvisionInput {
                name: Some("Replacement".into()),
                ..SsoProvisionInput::new("dup@acme.test")
            },
        )
        .await
        .unwrap();

        assert_eq!(first.user_id, second.user_id);
        assert_eq!(second.name.as_deref(), Some("Original Name"));
        assert_eq!(state.users.read().await.all_users().len(), 1);
    }

    /// Why: "any valid SSO email" means any address on the tenant's domains —
    ///      an address from somewhere else is not a login this tenant can vouch
    ///      for, and silently provisioning it would let anything through.
    /// Decision: reject rather than provision.
    #[tokio::test]
    async fn refuses_an_email_outside_the_tenant_domains() {
        let state = make_state().await;
        let err = provision_sso_user(
            &state,
            &sso_tenant(),
            SsoProvisionInput::new("x@other.test"),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, EmulatorError::ValidationError(_)));
    }

    /// Why: a tenant can opt out of provisioning, and then an unknown user must
    ///      keep getting the pre-existing "user not found" answer.
    /// Decision: honor `enabled: false` inside provisioning itself, so no caller
    ///      can bypass it.
    #[tokio::test]
    async fn refuses_when_the_tenant_disables_provisioning() {
        let state = make_state().await;
        let tenant = Tenant {
            sso_jit_provisioning: Some(crate::types::SsoJitProvisioning {
                enabled: false,
                ..Default::default()
            }),
            ..sso_tenant()
        };

        let err = provision_sso_user(&state, &tenant, SsoProvisionInput::new("nope@acme.test"))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::UserNotFound));
    }

    /// Why: role assignment is the main thing a developer wants to control for a
    ///      throwaway SSO user, and it can come from three places.
    /// Decision: explicit input wins over the tenant defaults, which win over
    ///      the role store's default roles.
    #[tokio::test]
    async fn role_precedence_is_input_then_tenant_then_store_defaults() {
        let state = make_state().await;
        {
            let mut roles = state.roles.write().await;
            roles
                .create("Member".into(), "store default".into(), vec![])
                .unwrap();
            roles.set_default("Member", true).unwrap();
        }

        let tenant_with_defaults = Tenant {
            sso_jit_provisioning: Some(crate::types::SsoJitProvisioning {
                default_role_names: vec!["Tenant Default".into()],
                ..Default::default()
            }),
            ..sso_tenant()
        };

        let from_store =
            provision_sso_user(&state, &sso_tenant(), SsoProvisionInput::new("a@acme.test"))
                .await
                .unwrap();
        assert_eq!(from_store.role_names, vec!["Member".to_string()]);

        let from_tenant = provision_sso_user(
            &state,
            &tenant_with_defaults,
            SsoProvisionInput::new("b@acme.test"),
        )
        .await
        .unwrap();
        assert_eq!(from_tenant.role_names, vec!["Tenant Default".to_string()]);

        let from_input = provision_sso_user(
            &state,
            &tenant_with_defaults,
            SsoProvisionInput {
                role_names: vec!["Chosen".into()],
                ..SsoProvisionInput::new("c@acme.test")
            },
        )
        .await
        .unwrap();
        assert_eq!(from_input.role_names, vec!["Chosen".to_string()]);
        assert_eq!(
            from_input.user_tenants[0].role_names,
            vec!["Chosen".to_string()]
        );
    }

    /// Why: the config screen posts free-text, and a typo like a bare username
    ///      must not create a user with a nonsense login ID.
    /// Decision: validate the address shape before anything is stored.
    #[tokio::test]
    async fn refuses_a_value_that_is_not_an_email_address() {
        let state = make_state().await;
        let err = provision_sso_user(&state, &sso_tenant(), SsoProvisionInput::new("just-a-name"))
            .await
            .unwrap_err();
        assert!(matches!(err, EmulatorError::ValidationError(_)));
    }

    /// Why: an identity provider asserting into its own tenant is authoritative
    ///      for whoever it names, and such a tenant often declares no domains at
    ///      all — enforcing an empty list would block every IdP sign-in.
    /// Decision: only check domain ownership when the tenant declares domains.
    #[tokio::test]
    async fn a_tenant_with_no_domains_accepts_any_asserted_address() {
        let state = make_state().await;
        let tenant = Tenant {
            id: "idp-only".into(),
            name: "IdP Only".into(),
            auth_type: AuthType::Saml,
            ..Default::default()
        };
        state.tenants.write().await.insert(tenant);

        let user = provision_sso_user_for_tenant(
            &state,
            "idp-only",
            SsoProvisionInput::new("anyone@anywhere.test"),
        )
        .await
        .unwrap();
        assert_eq!(user.user_tenants[0].tenant_id, "idp-only");
    }

    #[test]
    fn display_name_splits_common_local_part_separators() {
        assert_eq!(display_name_from_email("jane.doe@x.test"), "Jane Doe");
        assert_eq!(
            display_name_from_email("ada_lovelace@x.test"),
            "Ada Lovelace"
        );
        assert_eq!(display_name_from_email("grace@x.test"), "Grace");
    }

    #[test]
    fn normalize_email_rejects_malformed_values() {
        assert_eq!(
            normalize_email(" User@Example.test "),
            Some("user@example.test".to_string())
        );
        assert_eq!(normalize_email("no-at-sign"), None);
        assert_eq!(normalize_email("@example.test"), None);
        assert_eq!(normalize_email("user@nodot"), None);
    }
}
