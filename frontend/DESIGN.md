# Appearance and interaction system

The app keeps Vue, Quasar, Pinia, existing routes and domain rules. CSS tokens
live in `src/css/app.scss`; Quasar's `--q-*` variables and existing Markdown,
highlight and chart variables remain supported aliases.

## Appearance contract

`theme` stores the user's intention (`system`, `light`, `dark`). The optional
`appearance` object stores `theme_id` (`orbit`, `linen`, `forest`), `density`
(`comfortable`, `compact`) and `reduced_motion` (boolean). Older preferences
receive Orbit, comfortable density and the OS motion preference by default.
Rust validates enum values and merges partial appearance patches without
replacing siblings, notifications or regional preferences. Storage is the
existing JSONB `users.preferences`; no schema migration is necessary.

The theme store applies `data-theme`, `data-theme-id`, `data-density` and
`data-motion` to `<html>`, synchronizes Quasar Dark and listens to the OS.
Charts observe both mode and identity. CSS also respects OS reduced motion.
Local appearance caches are versioned and scoped by authenticated account;
only anonymous/portal caches are used before authentication. Cache failures
fall back to defaults. Portal appearance is local and separate.

Settings previews are temporary. Save confirms server data; cancel/leave
restores confirmed appearance and language. A failed save keeps an explicitly
unsaved preview for retry. Other settings tabs/reload are disabled during an
unsaved appearance edit so another response cannot overwrite that draft.

## Tokens

| Family   | Main tokens                                                                                                       |
| -------- | ----------------------------------------------------------------------------------------------------------------- |
| Surfaces | `--surface-page`, `--surface-nav`, `--surface-panel`, `--surface-raised`, `--surface-hover`, `--surface-selected` |
| Text     | `--text-primary`, `--text-secondary`, `--text-disabled`, `--text-accent`, `--text-on-action`                      |
| Actions  | `--action-bg`, `--action-hover`, `--action-pressed`, `--focus-ring`                                               |
| Borders  | `--border-subtle` for structure; `--border-control` for controls                                                  |
| State    | `--success-*`, `--warning-*`, `--error-*`, `--info-*` backgrounds/text                                            |
| Metrics  | 4/8/12/16/24/32px spacing; control/panel/dialog radii; control/row heights; reading width; header height          |

Text color and font weight are independent. Primary actions use white text on
the action background; links/flat actions use the separate accent text token.
User tag backgrounds are preserved; text uses sRGB relative luminance to pick
black or white. Comfortable controls remain the mobile default, with 44px
button targets. Respect zoom and avoid decorating passive content as clickable.

## Shared behavior

`PageHeader` gives pages a consistent hierarchy. `EmptyState` distinguishes
empty, no results and error. DataTable uses horizontal separators, explicit
open actions, contextual selection, permanently available create action,
applied filter chips and independent filter drafts. Long text opens a popover.
Number columns align right. Table writes are serialized and become confirmed
only after success; logout invalidates queued writes and cached table state.

Kanban exposes move menus using the same move service as dragging, announces
results and preserves rollback/domain validation. Mobile shows a selected
state instead of four consecutive columns. Cards show real priority/due dates;
tags are absent from the task summary API and are not fabricated.

Form/detail pages share surface metrics and form actions. Task/project forms
separate essential identification from planning/billing/content. Date controls
are buttons. Markdown and attachments inherit appearance. Draft keys freeze
account/record identity, preventing a route leave/logout from writing under a
different key. Invoice/payment/change-request forms reuse the same draft and
leave protection. Browser File objects are not persisted in local drafts.
Presets confirm only after the API succeeds and keep retry dialogs open on error.

## Verification

Use `scripts/with-resource-limits.py` for Linux checks; Cargo defaults to two
jobs. Check frontend with `npm run lint`, `npx vue-tsc --noEmit -p tsconfig.json`
and `npm run build`, sequentially. Rust contract tests use `cargo test --locked`.
Headless `UI_VERIFY=1 node scripts/capture-ui.mjs` verifies appearance, failures,
contrast and reflow; `UI_VERIFY_FORMS=1` checks drafts, failed presets and real task create/edit (then deletes the temporary task).
`UI_VERIFY_WORKFLOWS=1` adds keyboard, table persistence/pagination, regional
financial forms and real attachment upload/download/delete with disposable records. These opt-in
checks require the dedicated `ui-tests@horbyta.test` account. Artifacts and
credentials remain outside Git in `target/ui-tests/`. See root `Plans.md` for
phase status, evidence and remaining migration work.

## Dashboards and client portal

Dashboards separate summary, trends and follow-up. Passive KPI cards have no
hover movement. Navigation is an explicit labelled action to an existing list;
it does not imply that a matching filter has already been applied.

Charts retain their lazy registration, register ECharts ARIA and scrollable
legends, and provide native expandable data tables outside the canvas. Monthly
revenue sorts raw dates before formatting, keeps currencies separate and uses
stable identity colors plus symbols/line styles. Data tables reproduce plotted
zero values. Overview refresh updates unfiltered cards; filtered cards reload
their confirmed query. Failed filter requests preserve the draft dialog, previous
data and confirmed filters. Theme identity/mode and reduced motion are reactive.

Portal appearance uses its own local scope. Client table state is versioned and
scoped by owner/client/table, uses browser storage and never calls professional
UI-state endpoints. Portal date/number formatting uses public defaults rather
than private professional preferences. Client request drafts have a frozen client
identity; locked requests do not restore editable drafts. Backend permissions,
optimistic versions and credential/token flows retain their existing contracts.

Login, registration and client login share an accessible responsive frame with
language/appearance controls, password-manager autofill, focused fields and
inline error alerts. No new raster assets or essential external fonts are added.
The PWA manifest matches the initial Orbit palette; the document theme-color
follows the resolved navigation surface, including system-mode changes.

`UI_VERIFY_DASHBOARD_PORTAL=1` adds financial browser fixtures (never ledger
writes), actual disposable client credentials/token login and portal records,
regional chronology, filter failures/refresh, theme persistence, account isolation,
2FA UI challenge/error/backup-code contracts and ES/EN/PT font fallback captures.
2FA challenge responses are simulated; this check does not certify backend TOTP
cryptography. Use the same resource wrapper and dedicated account as above.

`UI_VERIFY_CLIENT_STATES=1` verifies portal detail hierarchy/contrast, submitted
and cancelled request UI, leave guards and draft restoration using browser-only
responses. It works without PostgreSQL and does not verify server authorization.
Saved evidence distinguishes these checks from the earlier real credentials/token
portal checks. The extended real-record suite passed after PostgreSQL access was restored;
root Plans.md preserves its evidence and the earlier interruption.

## Final migration review

`UI_VERIFY_MIGRATION=1` adds the 96 theme/mode/density/width combinations,
professional page families, text contrast and Chromium accessibility names,
regional/long content, sticky editor focus, touch emulation, modal keyboard and
real A/B account isolation. Temporary records are deleted and preferences restored.
`UI_VERIFY_CLIENT_STATES=1` covers 320/360/768/1440px in all three languages.
Browser fixtures test UI states; real portal credentials/token tests remain separate.

`UI_VERIFY_PRODUCTION=1` checks the compiled PWA with an isolated browser,
real API login, local fonts, global blocked Storage, anonymous translated reflow,
offline login shell and forward/backward modal focus cycles. Use the local review
server in `scripts/serve-ui-build.mjs` after building. Nginx remains the VPS server.
Development Devtools requires its own Storage; development checks block app keys
and production checks block all Storage reads/writes. No dependency is patched.

The final compatibility aliases are the used `--bg-*`, `--text-color`,
`--text-muted`, `--border-color`, `--card-bg`, plus Quasar, Markdown and chart
contracts. Four unused aliases were removed. Semantic spacing/action metrics stay
available as the design scale; they are not obsolete aliases. State utility text
uses `--success-text`, `--warning-text`, `--error-text` and `--info-text`, while
filled state backgrounds retain their Quasar colors. Headers and field messages
reference text tokens directly, including error labels.

Shared primitives: PageHeader, EmptyState, DataTable, AppearanceMenu, AuthFrame,
MarkdownEditor/Renderer/Toolbar, attachment lists, Kanban move controls, dashboard
KPI/chart cards and ChartDataTable. Each uses the same appearance contract; no
parallel global component framework or new image assets were introduced.

The evidence is a scoped engineering review, including spatial emulation of 200%
zoom. It is not a screen-reader, physical-device or global WCAG certification.
No participant-based task-speed improvement is claimed. Existing backend
notification-query errors remain outside the visual migration and are documented.

Modal keyboard navigation wraps at the first/last enabled visible control through
`utils/dialogFocus.ts`, installed by the appearance boot and cleaned up on app
unmount/HMR. Browser shortcuts and nonmodal popups retain their normal behavior.
Production checks include 40 Tab and 40 Shift+Tab steps, Enter opening and Escape
restoring the original trigger.

Final review passed 225 named browser checks, 96 matrix cells and 36 Rust tests.
The static entry gzip graph grew 3.67%; the observed cold login including startup
and dynamic UI imports grew 12.09% (25.66 KiB). Total build assets grew 2.19%, with
no increase in locally bundled fonts. Chart modules remain lazy before login;
Workbox still precaches them in the background. Sizes, captures, methods and
limitations are saved under `target/ui-tests/f10/` and root Plans.md.
