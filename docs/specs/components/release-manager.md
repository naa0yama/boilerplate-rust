# Release Manager

Self-managed GitHub Actions workflow that replaced `Songmu/tagpr` in this
repository. Maintains identical release UX while eliminating the dependency
on an external action that cannot handle `[workspace.package]` in Cargo.toml.

## Background

`tagpr` v1.19.0 added Cargo.toml support but only handles `[package]`, not
`[workspace.package]`. Each tagpr upgrade risked silent breakage. Past
incidents included version bump failures, repeated release PRs, and a
required 120s polling workaround.

## File Layout

| Path                                     | Role                                                           |
| ---------------------------------------- | -------------------------------------------------------------- |
| `.github/workflows/release-manager.yaml` | Main workflow (replaces `tagpr.yaml`)                          |
| `.github/workflows/release.yaml`         | Build + upload; called unchanged via `workflow_call`           |
| `.github/release-manager-pr-template.md` | PR body template (placeholders substituted via bash expansion) |
| `.github/release.yml`                    | GitHub Release Notes category config                           |

Deleted: `.tagpr`, `.tagpr-version-bump.sh`, `.mise/config.tagpr.toml`

## Workflow Triggers

```yaml
on:
  push:
    branches: [main]
  pull_request:
    types: [labeled, unlabeled]
    branches: [main]
  workflow_dispatch: {}
```

`pull_request` activates when `tagpr:minor` or `tagpr:major` is added/removed
on the `release/next` PR. The `prepare-pr` job's `if:` guard filters to only
the `release/next` head ref.

## Job Graph

```
detect           (push events only)
  └── outputs: is_release_merge, release_tag, skip

prepare-pr
  └── if: (push && !is_release_merge && !skip) OR pull_request on release/next
  └── steps: version-bump, changelog, commit via REST API, create/update PR

create-tag       (push events only)
  └── if: is_release_merge == true && !skip
  └── steps: create tag via REST API, delete release/next branch
  └── outputs: tag

release
  └── needs: create-tag
  └── if: tag != ''
  └── uses: ./.github/workflows/release.yaml
```

## Key Design Decisions

### Release detection (commit message pattern)

The `detect` job matches the merge commit message against `^Release for v`
and cross-checks against `Cargo.toml` version. This avoids the GitHub
commit-to-PR index lag that required the 120s polling workaround in tagpr.

### Commit strategy: REST API blob → tree → commit

All automated commits use the GitHub REST API Git Database endpoints, not
`git commit` + push (which would be unsigned and fail branch protection), and
not the `createCommitOnBranch` GraphQL mutation (which bundles all files in
one request body and fails when Cargo.lock + CHANGELOG.md exceed a few MB).

REST API per-file blob upload has a 100MB per-file limit. Commits created
server-side via `POST /git/commits` with GITHUB_TOKEN appear as "Verified".

### Branch name

`release/next` (no version in name) so that bump-type label changes can be
applied without closing/reopening the PR.

### Version bump

Reads `CURRENT` from `main` HEAD (`actions/checkout ref: main`) to prevent
double-bump on idempotent re-runs. `sed` targets `^version = "X.Y.Z"` at
`[workspace.package]` (verified to be the first match in this Cargo.toml).

### Changelog generation

GitHub Release Notes API (`POST /repos/{owner}/{repo}/releases/generate-notes`)
respects the existing `.github/release.yml` category configuration.

### Tag creation

Uses `POST /git/refs` only — lightweight tag pointing directly at the signed
merge commit. The commit's web-flow "Verified" status carries through to the
tag ref without any GPG key setup (identical to tagpr v0.1.14 behaviour).

Annotated tags (`POST /git/tags`) were dropped because GITHUB_TOKEN cannot
GPG-sign them, which caused the tag to appear "Unverified" in the GitHub UI.

## Idempotency Invariants

- Double-run on the same push: no duplicate PR, no duplicate tag, no error.
- Label bump then re-push: `prepare-pr` always reads `CURRENT` from `main`,
  so the calculation is not affected by prior `release/next` content.
- Label removal: reverts version to patch; PR title/body update.
- Tag creation race: second run hits the 404→skip guard.

## Concurrency

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false # tag creation must not be cancelled
```

`prepare-pr` has an additional **job-level** concurrency group
`release-manager-prepare-pr` (cancel-in-progress: false). The workflow-level
group includes `github.ref`, so push and pull_request events land in separate
groups; without the job-level group, two concurrent `prepare-pr` runs could
race to force-update `release/next`.

## Bump Labels

| Label         | Effect               |
| ------------- | -------------------- |
| `tagpr:minor` | Minor version bump   |
| `tagpr:major` | Major version bump   |
| (none)        | Patch bump (default) |

Label names kept identical to tagpr for backward compatibility.

## Deployment Checklist

For each new project rolling out release-manager:

- [ ] All crates under `crates/` use `version.workspace = true` (no own `version = "..."`)
- [ ] Root `Cargo.toml` has `[workspace.package] version = "X.Y.Z"`
- [ ] Guard passes on first `prepare-pr` run (check job logs)
- [ ] Tag ruleset `Restrict updates` / `Restrict deletions` — ON recommended
- [ ] Tag ruleset `Require signed commits` — **OFF** (lightweight tag inherits commit signature; cannot sign the ref itself)
- [ ] If using release immutability ruleset, confirm first publish succeeds before enabling
- [ ] `release/next` branch must NOT have `Restrict deletions` ruleset (create-tag deleteRef must work)

## Design Decisions Log

| Fix    | Decision                                                                                   |
| ------ | ------------------------------------------------------------------------------------------ |
| Fix 2  | `git describe` replaced by `gh api releases/latest` — avoids shallow clone failures        |
| Fix 3  | Job-level concurrency group on prepare-pr serialises push vs PR event runs                 |
| Fix 5  | `cargo update --workspace` instead of `generate-lockfile` — updates only workspace members |
| Fix 7  | `<!-- release-manager:notes -->` marker preserves user-editable PR body content            |
| Fix 9  | `gh label create --force` on each prepare-pr run — idempotent label bootstrap              |
| Fix 10 | Guard rejects crates with own `version = "..."` — unsupported by workspace-based bump      |
| Fix 11 | Lightweight tag (ref → commit SHA) restores "Verified" status lost with annotated tags     |
| Fix 12 | 403/422 tolerance on deleteRef; published-Release detection skips redundant release job    |
