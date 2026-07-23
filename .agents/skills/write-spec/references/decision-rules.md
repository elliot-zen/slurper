# Specification Decision Rules

Use these rules when deciding whether to update an existing feature or create a
new specification.

## Feature Ownership

A feature owns behavior when it is responsible for the same user goal, product
workflow, domain responsibility, or externally visible contract.

Do not rely only on wording. A request may belong to an existing feature even
when it uses different terminology.

Look for overlap in:

- User intent
- Product responsibility
- Page or workflow
- Public API or command behavior
- Domain state and transitions
- Permissions and validation rules
- Failure behavior

## Update an Existing Feature

Update an existing specification when the request:

- Extends an existing capability.
- Changes or corrects behavior already described by the feature.
- Fixes a bug in the feature.
- Adds another successful or failure path to the same user goal.
- Modifies an existing page, command, API, event, or workflow.
- Clarifies an omitted boundary, permission, validation rule, or edge case.
- Refactors or replaces the implementation without changing feature ownership.

For a bug fix, update the specification of the feature that should have produced
the correct behavior. Do not create a separate bug specification.

## Create a New Feature

Create a new specification only when all of the following are true:

1. The request introduces a distinct product capability or responsibility.
2. No existing feature clearly owns the behavior.
3. Adding the behavior to an existing specification would blur its boundaries.
4. The new capability is expected to remain meaningful beyond the current issue
   or implementation.

Creating a new specification should be uncommon for bug fixes.

## Feature IDs

Use a stable lowercase `kebab-case` ID, for example:

```text
user-login
order-refund
repository-sync
notification-settings
```

The ID must:

- Describe a product capability rather than an implementation.
- Use lowercase letters, numbers, and hyphens.
- Remain stable across implementation changes.
- Avoid issue numbers, dates, temporary project names, and internal component
  names when possible.

## Conflicts

When a request conflicts with an existing specification:

1. Identify the conflicting product rule or technical constraint.
2. Determine whether the request intentionally changes product behavior.
3. Update `PRODUCT.md` to state the new intended behavior and boundaries.
4. Update `TECH.md` with compatibility, migration, rollout, and rollback impact.
5. Record the intentional behavior change in `HISTORY.md`.

Do not silently preserve both contradictory behaviors.

## Specification and Implementation Disagreement

When implementation behavior disagrees with the specification, treat the
specification as the intended behavior unless the user explicitly confirms that
the implementation should become the new product contract.

When the implementation becomes the intended contract, update the specification
before modifying more code.
