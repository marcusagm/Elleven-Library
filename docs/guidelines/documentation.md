# 📚 Documentation Standards

High-quality documentation is as important as high-quality code. This guide covers how to write effective documentation for **Mundam**.

---

## 📝 General Principles

1.  **Audience First**: Write for the person who will read it. Are they a new contributor? An end-user? A maintainer?
2.  **Keep it Fresh**: Outdated documentation is worse than no documentation. Update docs in the same PR that changes the code.
3.  **Examples**: Always provide examples. Code snippets are worth a thousand words.

---

## 🔧 Code Documentation

### JavaScript/TypeScript (TSDoc)

Use TSDoc style comments (`/** ... */`) for exported functions, classes, and interfaces. All comments must be in **English**.

**CRITICAL RULE:** Never use visual separators (e.g., `// ============================` or `// --- Helpers ---`) to divide files into sections. If a file requires visual boundaries to be readable, it has too many responsibilities and should be broken down into smaller files or functions. Avoid inline comments explaining _what_ code does; instead, extract the logic into functions with descriptive names.

#### Template for Solid.js Components

```tsx
/**
 * Description of the UI component and its primary responsibility.
 *
 * @param {ComponentProps} props - Component properties.
 * @returns {JSX.Element} The rendered component.
 *
 * @example
 * <Button variant="primary" onClick={handleClick}>Click Me</Button>
 */
```

#### Template for Functions / Hooks

```ts
/**
 * A complete description of what the function or custom hook does.
 *
 * @param {Type} parameterName - Description of the parameter.
 * @param {Type} [optionalParameterName] - Description of an optional parameter.
 * @returns {Type} Description of the value returned by the function.
 * @throws {ErrorType} Description of potential errors thrown.
 *
 * @example
 * const ratio = calculateAspectRatio(1920, 1080); // Returns 1.777...
 */
export function calculateAspectRatio(width: number, height: number): number { ... }
```

#### Template for Interfaces / Types / Variables

```ts
/**
 * A complete explanation of the interface or type purpose.
 */
export interface EntityProps {
    /** Short description of the property's responsibility */
    propertyName: string;
}
```

#### Template for Objects, State Proxies and Hook Return Accessors

When avoiding visual separators inside hook implementations, properties or actions nested within standard return objects must be properly documented.

```ts
/**
 * Hook providing access to filter state and actions for the application content.
 *
 * @returns {Object} Accessors and methods for filtering and sorting items.
 */
export const useFilters = () => {
    return {
        /** Currently selected tag names */
        get selectedTags() {
            return filterState.selectedTags;
        },
        /** Unique identifiers for configured folders */
        get selectedFolderId() {
            return filterState.selectedFolderId;
        },
        /** Toggle active statuses */
        toggleTag: withRefresh(filterActions.toggleTag)
    };
};
```

### Rust (Rustdoc)

Use triple slash (`///`) for doc comments on public items.

- Use generic Markdown for formatting.
- Include `# Examples` sections which are automatically tested via `cargo test --doc`.

```rust
/// resizing an image to fit within a bounding box.
///
/// # Arguments
///
/// * `img` - The source image buffer.
/// * `max_dim` - The maximum dimension (width or height).
///
/// # Returns
///
/// A new image buffer resized to fit.
pub fn resize_image(img: &DynamicImage, max_dim: u32) -> DynamicImage { ... }
```

---

## 📖 README Files

Each major directory (`src-tauri/src/*` or modules) should ideally have a `README.md` if the complexity warrants it.

**Structure of a good README:**

1.  **Title & Description**: What is this module?
2.  **Usage**: How do I use it?
3.  **Configuration**: What options are available?
4.  **Troubleshooting**: Common pitfalls.

---

## 📂 Project Docs (`/docs`)

- **Plans**: Use `docs/plans/` for rigorous technical planning (RFCs).
- **Ideas**: Use `docs/idea/` for brainstorming.
- **Reports**: Use `docs/report/` for post-mortems or analysis.
