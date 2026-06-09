## Context

Four small gaps in user management API parity. All are thin route handlers that delegate to existing UserStore methods or trivial extensions.

## Approach

1. **Invited status** — extend `status_update` match arm to accept `"invited"` and just set `user.status = "invited"`. No store method change needed — we directly mutate the user.
2. **Add role** — new handler `add_roles` that reads current roles, merges the new ones (deduplicating), and writes back. Reuses `set_roles_for_user` in the store.
3. **Update picture** — new handler that accepts `{ loginId, picture }` and calls `user_store.update()` with a `UserUpdate` containing only the picture field.
4. **Update custom attribute** — new handler that accepts `{ loginId, attributeKey, attributeValue }` and merges into the user's custom_attributes map.

All four follow the existing handler pattern: deserialize → auth check → mutate store → return `{ user: ... }`.
