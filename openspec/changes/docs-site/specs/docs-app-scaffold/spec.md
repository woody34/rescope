## ADDED Requirements

### Requirement: Astro/Starlight documentation site
The docs site SHALL be an Astro + Starlight application at `apps/docs/` with MDX-based content pages, sidebar navigation, and the Rescope dark aesthetic.

#### Scenario: Docs site builds and serves
- **WHEN** `npm run build` is run in `apps/docs/`
- **THEN** the app SHALL produce a static build output that can be served by any static file host

#### Scenario: Navigation between pages
- **WHEN** a user clicks a sidebar navigation link
- **THEN** the app SHALL navigate to the corresponding documentation page

#### Scenario: MDX content renders
- **WHEN** a documentation page is loaded
- **THEN** the MDX content SHALL render as styled HTML with code syntax highlighting, headings, and the Rescope dark theme

### Requirement: Documentation content pages
The docs site SHALL include initial content pages covering Getting Started, API Reference overview, and at least one guide.

#### Scenario: Getting Started page
- **WHEN** a user navigates to the Getting Started page
- **THEN** the page SHALL explain installation, configuration, and basic usage of Rescope

#### Scenario: Guide page
- **WHEN** a user navigates to a guide (e.g., "OTP Authentication")
- **THEN** the page SHALL walk through the auth flow with code examples and explanations

### Requirement: Responsive layout
The docs site SHALL have a responsive layout with Starlight's built-in sidebar (visible on desktop, collapsible on mobile).

#### Scenario: Desktop layout
- **WHEN** the viewport is ≥ 768px
- **THEN** the sidebar SHALL be persistently visible alongside the content area

#### Scenario: Mobile layout
- **WHEN** the viewport is < 768px
- **THEN** the sidebar SHALL be hidden by default and toggleable via a menu button
