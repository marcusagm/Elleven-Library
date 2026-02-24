# 🎨 Frontend Guidelines (Solid.js + TypeScript)

This document outlines the coding standards, best practices, and architecture specific to the **Mundam** frontend, built with [Solid.js](https://www.solidjs.com/) and [TypeScript](https://www.typescriptlang.org/).

---

## 🏗️ Architecture & Component Structure

### Component Colocation

We follow a feature-based architecture. Components, their styles, and specific utilities should be colocated.

```
src/
  components/
    features/
      VideoPlayer/
        VideoPlayer.tsx      # Main component
        VideoControls.tsx    # Sub-component
        videoPlayer.css      # Component-specific styles
        utils.ts             # Private utilities
```

### Solid.js Specifics

#### 1. Reactivity & Signals

- **Never destructure props** passed to components unless you wrap them in `splitProps`. Destructuring breaks reactivity in Solid.
- Use `createMemo` for derived state to prevent unnecessary re-calculations.
- Use `createEffect` sparingly. Prefer derived signals (`createMemo`) over synchronizing state with effects.

```tsx
// ✅ Correct
const Component = (props: { title: string }) => {
    return <h1>{props.title}</h1>;
};

// ❌ Avoid (Breaks reactivity)
const Component = ({ title }) => {
    return <h1>{title}</h1>;
};
```

#### 2. Control Flow

Use Solid's built-in control flow components (`<Show>`, `<For>`, `<Switch>`) instead of array maps or ternary operators for better performance and readability.

```tsx
// ✅ Correct
<Show when={!props.loading} fallback={<Loader />}>
    <For each={props.items}>{item => <ItemView item={item} />}</For>
</Show>;

// ❌ Avoid
{
    !props.loading ? props.items.map(item => <ItemView item={item} />) : <Loader />;
}
```

---

## 📝 Coding Standards

### Naming Conventions

- **Never abbreviate variable names.**
  Each variable name must describe **exactly** its responsibility.

    ```ts
    // ✅ Correct
    const circuitComponentList = [];

    // ❌ Avoid
    const compList = [];
    ```

- **No Single/Two-letter names:** Avoid `i`, `j`, `dx`, `dy`. Use `index`, `deltaX`, `deltaY`.
- **Components:** PascalCase (e.g., `VideoPlayer.tsx`)
- **Signals:** camelCase, preferably describing the data (e.g., `const [isActive, setIsActive] = createSignal(false)`)
- **Event Handlers:** Prefix with `handle` (e.g., `handleClick`, `handleInputChange`)
- **Props:** camelCase (e.g., `isOpen`, `hasError`).

### Formatting Rules

These are enforced automatically via ESLint and Prettier (`.prettierrc`):

| Rule                       | Description                           |
| -------------------------- | ------------------------------------- |
| **4 spaces**               | Indentation (no tabs)                 |
| **Single quotes `'`**      | For strings                           |
| **Semicolons**             | Required at the end of each statement |
| **Trailing commas**        | None (as per `.prettierrc`)           |
| **Newline at EOF**         | Always required                       |
| **No trailing spaces**     | On any line                           |
| **One space after commas** | Consistent spacing                    |

### Coding Principles

- **Single Responsibility Principle (SRP):**
  Each function or component must have **only one clear purpose**.

- **No Visual Section Separators:**
  **Never** add comments to divide the file into sections (e.g., `// === State ===`, `// --- Helpers ---`, etc.). If a file needs visual sections, it indicates excessive responsibilities. Extract the logic into descriptive functions or separate files.

- **Self-Documenting Code over Inline Comments:**
  Avoid inline comments that explain _what_ the code is doing (e.g., `// calculates the columns that fit`). Instead, extract that logic into a well-named function (e.g., `calculateFittingColumns(...)`). Write comments only to explain _why_ something is done or to provide TSDoc annotations.

- **Readability over cleverness:**
  Favor code that is **easy to understand** over complex or compact solutions.
    - **Avoid nested ternaries**.
    - **Strict Equality:** Always use `===` or `!==`.

- **Avoid side effects:**
  Functions should not unexpectedly modify global variables or unrelated states.

- **Avoid deeply nested conditionals:**
  Refactor complex logic into smaller, testable functions or use early returns.

- **Explicit Returns:**
  Every function should clearly define what it returns.

### TypeScript

- **No `any`**: Avoid `any` at all costs. Use `unknown` or specific types.
- **Interfaces over Types**: Use `interface` for object definitions and `type` for unions/intersections.
- **Strict Null Checks**: Maintain strict null checks. Handle `null` and `undefined` explicitly.

```ts
// ✅ Correct
interface VideoProps {
    src: string;
    onPlay?: () => void;
}

// ❌ Avoid
type VideoProps = {
    src: string;
    onPlay: any;
};
```

### Code Complexity

| Metric              | Limit | Enforcement              |
| ------------------- | ----- | ------------------------ |
| Function complexity | 10    | ESLint `complexity` rule |
| Max lines per file  | 400   | ESLint `max-lines` rule  |

If you exceed these limits, consider **splitting** logic into smaller functions or composables.

---

## 💅 Styling

- **Tokens**: Always use design tokens from `src/styles/tokens.css` via CSS variables.
- **Scoped CSS**: Use CSS Modules or straightforward class naming BEM-like if standard CSS is used to avoid collisions.
- **No Hardcoded Values**: Avoid magic numbers (pixels, hex colors) in the CSS file. References variables like `var(--color-bg-surface-1)`.

---

## ⚙️ Best Practices

### ✅ Do

- Use **`const`** and **`let`**, never `var`.
- Use **ES Modules** (`import` / `export`) consistently.
- Prefer **pure functions** and **immutable data structures**.
- Write **clear, concise comments** explaining _why_ — not _what_.

### ❌ Don’t

- Leave unused variables or imports.
- Commit commented-out code blocks.
- Use `console.log()` for debugging — use `console.warn` or `console.error` if necessary.
- Push code containing `TODO` or `FIXME` notes without resolving them.
- Introduce “magic numbers” — define them as named constants (or use design tokens).

---

## 🧼 Linting

Run the linter before pushing:

```bash
npm run lint
```
