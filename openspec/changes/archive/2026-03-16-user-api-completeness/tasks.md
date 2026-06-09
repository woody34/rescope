## 1. Invited Status

- [x] 1.1 Extend `status_update` handler to accept `"invited"` status and set `user.status = "invited"`
- [x] 1.2 Unit test: `status_update_accepts_invited`
- [x] 1.3 Integration test: `POST /v1/mgmt/user/update/status` with `invited` returns 200 and user has status `invited`

## 2. Add Role Endpoint

- [x] 2.1 Add `add_roles` store method to `UserStore` — appends roles, deduplicating
- [x] 2.2 Add `add_roles` handler in `user.rs` accepting `{ loginId, roleNames }`
- [x] 2.3 Wire route: `POST /v1/mgmt/user/update/role/add`
- [x] 2.4 Unit test: `add_roles_appends_without_duplicates`
- [x] 2.5 Integration test: add role via API, verify user has both old and new roles

## 3. Update Picture Endpoint

- [x] 3.1 Add `update_picture` handler accepting `{ loginId, picture }`
- [x] 3.2 Wire route: `POST /v1/mgmt/user/update/picture`
- [x] 3.3 Unit test: `update_picture_sets_url`
- [x] 3.4 Integration test: set picture via API, load user, verify picture field

## 4. Update Custom Attribute Endpoint

- [x] 4.1 Add `update_custom_attribute` handler accepting `{ loginId, attributeKey, attributeValue }`
- [x] 4.2 Wire route: `POST /v1/mgmt/user/update/customAttribute`
- [x] 4.3 Unit test: `update_custom_attribute_merges_single_key`
- [x] 4.4 Integration test: set attribute via API, verify it appears in user's customAttributes
