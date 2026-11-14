# Roadmap

Updated 2026-11-13 against `main` at `5a42419` and the feature specifications
in `docs/features/`. Future work is tracked in GitHub Issues; this document is
the project-level index, while each feature spec remains the source of its
behavioral contract.

## Current state

| Area | State |
|---|---|
| Shared grocery list, item rules, product search and product choice | Implemented; remaining refinements are tracked below |
| Intake: Alexa, generic webhook, Google Tasks, triage | Implemented; live account/device acceptance remains |
| Order review, store-choice planner, delivery fee settings, purchase history and spending analysis | Implemented |
| Store handoff | Browser bookmark flow works for Woolworths and Coles; desktop native flow currently supports Woolworths |
| Desktop app | Tauri app, server connection, Woolworths fill/checkout, tray and notifications implemented |
| Auth, configuration, deployment and access logs | Implemented; see the listed hardening and retention follow-ups |
| Google Keep | Paused by decision because its unofficial master-token authentication is too risky |

## Roadmap issues

### Store access and ordering

- [#72 — Enable Coles login, trolley fill, and checkout in the desktop app](https://github.com/jesseross42/smart-home-groceries-listing/issues/72)
- [#70 — Read live delivery windows and fees from Woolworths and Coles](https://github.com/jesseross42/smart-home-groceries-listing/issues/70)
- [#74 — Add delivery constraints to order planning](https://github.com/jesseross42/smart-home-groceries-listing/issues/74)
- [#71 — Add one-tap order placement through the desktop app](https://github.com/jesseross42/smart-home-groceries-listing/issues/71)
- [#79 — Use member pricing and cross-product multibuys](https://github.com/jesseross42/smart-home-groceries-listing/issues/79)
- [#87 — Add a product specials preference to order planning](https://github.com/jesseross42/smart-home-groceries-listing/issues/87)
- [#78 — Import Woolworths and Coles order history](https://github.com/jesseross42/smart-home-groceries-listing/issues/78)
- [#80 — Add a product lookup screen](https://github.com/jesseross42/smart-home-groceries-listing/issues/80)
- [#91 — Store product and category data for spending analysis](https://github.com/jesseross42/smart-home-groceries-listing/issues/91)
- [#84 — Decide whether checkout extras are in scope](https://github.com/jesseross42/smart-home-groceries-listing/issues/84)

### Desktop app

- [#75 — Add optional desktop start at login](https://github.com/jesseross42/smart-home-groceries-listing/issues/75)

### Household workflows

- [#76 — Use product dislikes in automated order planning](https://github.com/jesseross42/smart-home-groceries-listing/issues/76)
- [#82 — Improve item rules and add a settings index](https://github.com/jesseross42/smart-home-groceries-listing/issues/82)

### Integration follow-ups and maintenance

- [#86 — Handle Google Tasks subtasks, notes, and disconnect revocation](https://github.com/jesseross42/smart-home-groceries-listing/issues/86)
- [#89 — Store the OpenAI triage key encrypted in application settings](https://github.com/jesseross42/smart-home-groceries-listing/issues/89)
- [#85 — Verify live store and voice integrations end to end](https://github.com/jesseross42/smart-home-groceries-listing/issues/85)

## Deliberately out of the roadmap

- **Google Keep intake:** paused; revisit only with an explicit decision to
  accept the unofficial library and master-token risks (`FEATURE_VOICE.md`).
- **Google Home shopping-list integration:** not supported by Google's public
  interfaces; the project uses Alexa, Google Tasks, or the generic webhook.
- **Automatic product suggestions or purchases:** excluded by the project's
  household-consent and no-upsell requirements.
- **Per-item ownership and roles:** not part of the shared household model.
- **Server-side store sessions:** the desktop app owns store authentication
  because the home server cannot complete the stores' sign-in flows.

## Tracking rule

Create a GitHub issue for future implementation, verification, or product
decision work and link it here. Keep completed behavior and remaining gaps in
the owning feature spec. Do not treat a paused or explicitly excluded item as
planned work without a new decision.
