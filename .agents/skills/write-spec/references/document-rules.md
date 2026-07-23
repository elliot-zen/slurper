# Specification Document Rules

## `specs/SPEC.md`

`SPEC.md` is the product feature index. It helps future agents find the feature
that owns a requested change.

Use this format:

```markdown
# Feature Specifications

| ID | Feature | Description |
| --- | --- | --- |
| user-login | User login | Allows users to authenticate and access their accounts. |
```

Each entry must contain:

- A stable feature ID.
- A concise feature name.
- A product-level description that distinguishes the feature from neighboring
  capabilities.

Do not include internal implementation details.

## `PRODUCT.md`

`PRODUCT.md` describes the complete intended feature from the user and product
perspective.

It must explain:

- The user problem and product value.
- Who uses the feature.
- Observable behavior.
- The normal successful user journey.
- Included and excluded responsibilities.
- Product rules, validation, permissions, and state transitions.
- User-visible failure behavior.
- Product-level compatibility expectations.

Do not include:

- Programming languages or frameworks.
- Classes, functions, internal services, or file paths.
- Database tables or storage internals.
- Queue, cache, deployment, or infrastructure details.
- Internal algorithms or implementation-only APIs.

Public APIs, commands, interfaces, and integrations may be described when they
are part of the user-visible contract.

For a bug fix, write the correct intended behavior. Do not preserve the buggy
behavior as the permanent specification.

## `TECH.md`

`TECH.md` describes the complete intended implementation after the change.

It may contain:

- Architecture and component responsibilities.
- File and module locations.
- Data models and migrations.
- APIs, commands, events, and configuration.
- Execution flows and state transitions.
- Validation and error handling.
- Security and authorization requirements.
- Concurrency, retries, and idempotency.
- Compatibility and migration constraints.
- Logging, metrics, traces, and audit records.
- Test strategy.
- Rollout and rollback plans.

For bug fixes, include:

- The technical root cause.
- The corrected implementation behavior.
- Affected components.
- Relevant edge cases.
- Regression-test requirements.

Do not use `TECH.md` as a chronological change log. Rewrite outdated sections so
the document reflects the intended current design.

## `HISTORY.md`

`HISTORY.md` is append-only.

Append exactly one line per change:

```text
<YYYY-MM-DD>: <change description> : <related issue or N/A>
```

The description must state the behavior or implementation outcome that changed.
Avoid vague entries such as:

- Updated spec
- Fixed issue
- Improved feature
- Refactored code

Use the supplied issue identifier or issue URL. Use `N/A` when no issue exists.
Never invent an issue reference.

Do not modify or remove previous entries unless the user explicitly requests
history cleanup.

## Document Responsibility Boundaries

- `SPEC.md` locates and summarizes features.
- `PRODUCT.md` defines product intent and observable behavior.
- `TECH.md` defines implementation design and operational requirements.
- `HISTORY.md` records when and why the specification changed.

Do not duplicate long explanations across documents. Put each statement in the
file that owns that responsibility.
