# Core Architecture Guidelines

This document defines the patterns and constraints for the Mundam core layer (Stores, Actions, and Domain Logic).

## 🛡️ 1. Action Pattern

All state mutations must occur through standardized actions. This ensures a unidirectional data flow and predictable state transitions.

### ActionResult and BaseError
Every action must return an `ActionResult<TData, TError>`.

```typescript
import { ActionResult } from '../types/actions';

export async function deleteAsset(id: string): Promise<ActionResult> {
  // logic...
}
```

## 🔍 2. Zod Integration & Validation

We use Zod to validate all payloads coming from the UI or external sources (Tauri, LocalStorage).

### Schema Location
- Each store should have a `schemas.ts` file in its directory.
- Example: `src/core/store/library/schemas.ts`.

### Naming Convention
- Schemas should be named with the `Schema` suffix.
- Example: `AddLocationSchema`.
- Payloads should be derived from schemas: `type AddLocationPayload = z.infer<typeof AddLocationSchema>`.

### Validation Policy
- Validate all payloads in the action layer using `Schema.safeParse()`.
- Convert Zod errors to `ActionResult` with `VALIDATION_ERROR` code.

## 🧱 3. Layer Constraints (Guardrails)

To maintain a clean architecture, the following constraints are enforced:

1.  **UI Isolation:** UI components (`src/components/`) must never call `tauriService` or backend APIs directly. They must go through Actions.
2.  **State Protection:** No component should import `setStore` or modify signals directly. Use exported actions.
3.  **No UI in Stores:** Stores and Actions must not import `toast` or other UI-specific utilities. Use Domain Events or return error states for the UI to handle.
4.  **No Circular Dependencies:** Stores should not import each other. Use a message bus or event dispatcher for cross-store communication.
