## ADDED Requirements

### Requirement: Auth method can be toggled off via Admin UI
The Auth Methods configuration page SHALL provide toggles (switches) for each authentication method. Disabling a method SHALL cause the emulator to reject auth requests for that method with a 4xx response.

#### Scenario: Disabling password auth causes signin to fail
- **WHEN** the Password auth method toggle is switched OFF via the Admin UI
- **THEN** `POST /v1/auth/password/signin` returns a 4xx response

#### Scenario: Disabling OTP auth causes OTP send to fail
- **WHEN** the OTP auth method toggle is switched OFF via the Admin UI
- **THEN** `POST /v1/auth/otp/signup/email` returns a 4xx response

---

### Requirement: Auth method can be re-enabled via Admin UI
Methods that have been disabled SHALL be re-enable-able via the same toggle control, restoring normal API behavior.

#### Scenario: Re-enabling password auth restores signin
- **WHEN** the Password auth method is disabled and then re-enabled via the Admin UI toggle
- **THEN** `POST /v1/auth/password/signin` returns HTTP 200 for valid credentials

---

### Requirement: Auth method toggle state persists across page reloads
The enabled/disabled state of each auth method SHALL be preserved by the emulator and reflected correctly when the Auth Methods page is reloaded.

#### Scenario: Disabled method still off after page re-navigation
- **WHEN** an auth method is toggled off and the user navigates away and back to the Auth Methods page
- **THEN** the toggle for that method is in the OFF position
