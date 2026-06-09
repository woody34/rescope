## ADDED Requirements

### Requirement: Curl test example on key creation
The system SHALL display a copy-ready curl command after a new access key is created. The curl command MUST use the actual cleartext key value and the emulator's host/port resolved from the current browser URL.

#### Scenario: Curl example displayed after key creation
- **WHEN** a new access key is created successfully
- **THEN** a curl command is displayed in a code block below the cleartext key, formatted as:
  ```
  curl -s -w '\n%{http_code}\n' <origin>/v1/mgmt/user/search \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer <cleartext>' \
    -d '{}' | jq
  ```

#### Scenario: Curl example uses dynamic emulator URL
- **WHEN** the emulator is running on `http://localhost:4600`
- **THEN** the curl command uses `http://localhost:4600` as the base URL (not hardcoded)

#### Scenario: Curl example uses actual key
- **WHEN** the key `K5c6e986a:b5d01268-1ea8` is created
- **THEN** the curl command contains `Authorization: Bearer K5c6e986a:b5d01268-1ea8`

#### Scenario: Copy curl command
- **WHEN** the user clicks the copy button next to the curl example
- **THEN** the full curl command is copied to the clipboard

### Requirement: Cleartext banner readability
The access key cleartext banner SHALL have sufficient color contrast for readability in the dark theme. The banner background, text, and code block colors MUST be updated to provide high contrast.

#### Scenario: Banner is readable in dark theme
- **WHEN** a new key is created and the cleartext banner is displayed
- **THEN** the banner text and code block have sufficient contrast against the background (no light-on-light text)

#### Scenario: Code block styling
- **WHEN** the cleartext key is displayed in the code block
- **THEN** the code block uses a dark background with light monospace text (consistent with the app's dark theme)
