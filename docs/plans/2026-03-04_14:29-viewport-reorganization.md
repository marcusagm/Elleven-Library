# Relatório de Reorganização do Viewport & Documentação de Atualizações 
(Data: 2026-03-04 14:29)

---

## Objective
Reorganize the `features/viewport` directory into a structurally logical layout, implement a composition-based `AssetCard` with dynamic metadata layouts (overlay vs. stacked), and provide an isolated UI preferences store to control card formatting and visible fields inside the grid view.

## 1. Directory Reorganization (`src/components/features/viewport/`)
Create thematic subdirectories to separate components:
- `assets/` 
  - Move `AssetCard.tsx`, `Thumbnail.tsx`, `thumbnail.css`. 
  - Create new composition components: `AssetItemContainer.tsx`, `AssetCardOverlay.tsx`, `AssetCardStacked.tsx`, `asset-card.css`.
- `layouts/`
  - Move `VirtualGridView.tsx`, `VirtualMasonry.tsx`, `VirtualListView.tsx`, `ListView.tsx`, `DragOverlay.tsx`, `EmptyState.tsx`, `empty-state.css`, `grid-view.css`, `list-view.css`.
- `toolbar/`
  - Move `ListViewToolbar.tsx`, `list-view-toolbar.css`.

Dependencies and imports will be updated systematically across the application.

## 2. Refactoring `AssetCard` (Composition / Render Props)
The `AssetCard.tsx` currently mixes DnD/Focus/Selection logic with HTML structure. We will split it into:

### `AssetItemContainer.tsx`
A logical "headless" component that handles:
- DnD integration (`useAssetDropZone`, `assetDragSource`)
- Virtual Focus synchronization
- ARIA properties and `gridcell` roles
- Native Event Management (Select, Context Menu, Double-click)
- Wraps the visual presentation logic by passing structured internal state properties downstream via Solid's `children/Render Props`.

### `AssetCardOverlay.tsx` and `AssetCardStacked.tsx`
Presentational ("dumb") components that render the metadata inside the card.
- `AssetCardOverlay`: Styles metadata over the thumbnail (hover-based), as current behavior.
- `AssetCardStacked`: Places metadata below the thumbnail.
- They iterate over active metadata fields configured in the store.

### `AssetCard.tsx`
Acts as the main facade that consumes the layout parameters (from preferences) and delegates rendering to `AssetItemContainer` embedding either `Overlay` or `Stacked` representation.

## 3. Viewport Preferences Store (`src/core/store/viewportPreferencesStore.ts`)
Create a new targeted Solid.js store for managing display settings independently of filter/search logic to conform with SRP.

**State Signature:**
```typescript
interface ViewportPreferencesState {
    metadataPosition: 'overlay' | 'stacked';
    visibleFields: MetadataField[];
}
type MetadataField = 'filename' | 'extension' | 'dimensions' | 'size' | 'rating' | 'modified_at' | 'created_at' | 'added_at' | 'tags';
```

## 4. Hook Facade & `ListViewToolbar` Updates
- Export `useViewportPreferences()` from `src/core/hooks/useViewportPreferences.ts`.
- Update `ListViewToolbar.tsx` to include popover sections to control `metadataPosition` and toggle `visibleFields` using the new hook.

## 5. Guidelines Compliance Checklist
- [x] No `any` types; strict `unknown` or discrete types.
- [x] Use `interface` primarily.
- [x] No variable abbreviations (e.g. `idx`, `evt`, `btn`).
- [x] TSDoc fully annotated over exported methods (`@param`, `@returns`).
- [x] Use PascalCase for components, camelCase for functions and files (except components).
- [x] No inline comments splitting files visually (`// === State ===`).
- [x] ARIA properly assigned in focusable nodes.
- [x] Reutilize `ListViewToolbar` components and icons accurately. 

---

## 6. Passo a Passo do que Foi Feito (Implementação Realizada)

Durante a execução deste plano, os seguintes passos foram tomados:

1. **Reorganização de Arquivos**: O diretório `features/viewport` foi limpo. Foram criadas as pastas `/assets`, `/layouts` e `/toolbar`, movendo seus respectivos arquivos logicamente para estes lugares. 
2. **Criação da Store de Preferências Visuais**: Foi implementada e documentada através do hook padronizado (`useViewportPreferences()`) e store `viewportPreferencesStore.ts`, suportando gerência de estado (local storage readiness/estrutura persistente) de quais metadados são sobrepostos no card, e como.
3. **Refatoração do Componente Viewport / Componentes Layout**:
    *  Correção extensa de caminhos de importação.
    *  Remoção progressiva de dependências legadas que apontavam para caminhos desatualizados (como os módulos de thumbnail, loader ou context de seleção).
4. **Refatoração de Assets / AssetCard (Headless + Facade)**:
    * `AssetItemContainer.tsx`: Foi criado como um componente "burro" logicamente denso focado na renderização do container Virtual com atributos ARIA, Foco, e DnD Native Events.
    * Criação do `AssetCardOverlay` e `AssetCardStacked` consumindo um map interno de `getFieldRenderValue`. Em vez do bloco massivo `switch/case`, um objeto padronizado foi usado para retornar JSX.
    * Alteração do `AssetCard.tsx` para se tornar um componente Facade simples que controla qual modelo renderizar dependendo das preferências estabelecidas pela store.
    * `asset-card.css` foi adicionado isolando lógicas atreladas à UI da grade principal.
5. **Correção de Tipagens do TS (Strict Mode)**:
    * Todo o processo de criação de componentes e loops do SolidJS que usavam argumentos implícitos `any` como `item` foram tipados como a interface `AssetItem`.
6. **Desacoplamento do ListViewToolbar**:
    * `ListViewToolbar.tsx` tinha crescido muito (problema de Lint `max-lines`). Ele foi fragmentado com sucesso em `HistoryNavigation.tsx`, `SortConfiguration.tsx` e `ViewConfiguration.tsx`. A orquestração deles foi deixada no arquivo pai que agora atende aos padrões rigorosos de linhas de código, e responsabilidade única.

## 7. Obstáculos Encontrados

*   **Tipagens Complexas Solid e Quebra de Referências Relativas:** Inicialmente as modificações estruturais do diretório impactaram fortemente e silenciosamente imports em múltiplas localizações da aplicação (`ListView.tsx`, `VirtualListView.tsx`). Houve erros frequentes sobre types implicitamente `any` durante loops do `<For>`.
*   **Problema de Propriedades de Estilização em Components SolidJS (HTML Attributes vs Virtual DOM)**: Ao definir opções estritas, as props customizadas injetavam classes indevidas. Tivemos que converter uma de `className` que costuma derivar de React-based props para a convenção nativa JSX de Solid `class`.
*   **Aviso de Limites Críticos de Arquivo (ESLint max-lines max: 300)**: Foi imposto um bloco quando o Toolbar cresceu devido aos múltiplos Tooltips e SelectOptions visuais. Isolamos de forma atômica para limpar os erros em etapa paralela.

## 8. Possíveis Melhorias Futuras (Futuro)

*   **Persistência em Disco Total:** Salvar ativamente a configuração de Store `ViewportPreferencesState` ao SQLite nativo via Tauri Commands do backend Rust assim que houver alteração visual, garantindo sessão imutável.
*   **Reaproveitamento de Formulários:** Transformar algumas lógicas expostas da `SortConfiguration` para reutilizar primitivos padronizados que criamos na documentação e design-system (ToggleGroups paramétricos com config prop-driven).
*   **Lazy Loading e Componentização Extra:** Avaliar o peso do Popover e Select com `lazy()` load. O menu Toolbar pode carregar subpaineis visuales assincronamente a depender do peso.
*   **Preview Interativo no Toolbar**: Melhorar o range do Slider para debouncing otimizado, evitando reflow do grid em tempo real massivo e controlando o render engine em computadores de baixo setup.
