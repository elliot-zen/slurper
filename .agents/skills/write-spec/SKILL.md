---
name: write-spec
description: Create and maintain product and technical specifications for features and bug fixes. Use before adding a feature, changing existing behavior, or fixing a bug. Always inspect the specification index first, update related specifications, and record the change history before implementation.
---

# Write Spec

Create or update specifications before implementing a feature, behavior change,
or bug fix.

Specifications use this structure:

```text
specs/
├── SPEC.md
└── <id>/
    ├── PRODUCT.md
    ├── TECH.md
    └── HISTORY.md
```

## Mandatory Rule

Before modifying implementation code:

1. Read `specs/SPEC.md`.
2. Search the index for related features.
3. Read the `PRODUCT.md`, `TECH.md`, and `HISTORY.md` of every potentially related feature.
4. Update an existing specification or create a new one.
5. Append the current change to `HISTORY.md`.

Never create a new specification without first checking whether an existing
feature owns the requested behavior.

Never begin implementation before the specification is updated.

## Workflow

### 1. Understand the change

Determine:

- The user-visible behavior being added, changed, or corrected.
- The feature area responsible for that behavior.
- The intended product boundaries and happy path.
- The affected technical components.
- The related issue identifier, when provided.

### 2. Inspect existing specifications

Read `specs/SPEC.md` and search by:

- Feature name
- User goal
- Domain concept
- Page, command, API, or workflow
- Bug symptom
- Related issue

When a related feature is found, read all three documents in its directory
before deciding how to proceed.

### 3. Select the specification

Update an existing feature when the request:

- Extends an existing capability.
- Changes behavior already owned by it.
- Fixes a bug in that behavior.
- Adds another path to the same user goal.
- Clarifies a boundary, rule, failure case, or edge case.
- Changes implementation without introducing a distinct product capability.

Create a new feature specification only when the request introduces a distinct
product capability or responsibility that cannot be represented clearly by an
existing feature.

Read `references/decision-rules.md` when ownership is ambiguous or the request
conflicts with an existing specification.

### 4. Update the documents

For an existing feature:

1. Update `PRODUCT.md` so it describes the complete intended product behavior.
2. Update `TECH.md` so it describes the complete intended implementation.
3. Append one new line to `HISTORY.md`.
4. Update `specs/SPEC.md` only when its summary is no longer accurate.

For a new feature:

1. Choose a stable, product-oriented, lowercase `kebab-case` ID.
2. Add it to `specs/SPEC.md`.
3. Create the feature directory.
4. Create `PRODUCT.md`, `TECH.md`, and `HISTORY.md` from the templates in `assets/`.

Read `references/document-rules.md` before writing or restructuring these files.

### 5. Validate before implementation

Verify that:

- `specs/SPEC.md` was read first.
- Related specifications were reviewed.
- No duplicate feature specification was created.
- `PRODUCT.md` contains no internal implementation details.
- `PRODUCT.md` explains boundaries and the user happy path.
- `TECH.md` contains actionable implementation details.
- Bug fixes include root-cause and regression-test coverage in `TECH.md`.
- `HISTORY.md` contains exactly one new correctly formatted entry.
- No issue identifier was invented.

## History Format

Append exactly one line per change:

```text
<YYYY-MM-DD>: <change description> : <related issue or N/A>
```

Example:

```text
2026-07-23: Fixed expired login links being accepted after account activation : ISSUE-187
```

Use the current date. Use `N/A` when no issue is provided.

## Output

After updating the specification, report:

- Whether an existing feature was updated or a new feature was created.
- The selected feature ID.
- Files created or modified.
- Main product behavior or boundary changes.
- Main technical changes.
- The history entry added.
- Any unresolved decision, risk, or missing issue reference.

Do not claim that files were changed unless they were actually written.
