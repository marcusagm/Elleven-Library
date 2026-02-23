# Frontend Code Quality — Refactoring & Type Safety Sprint

**Data:** 2026-02-23  
**Status:** ✅ Implementado e Verificado (`tsc --noEmit` → 0 erros)  
**Escopo:** Redução de complexidade, eliminação de `any`, remoção de `console.log`, extração de módulos

---

## Contexto

O frontend do Mundam acumulou diversos problemas de qualidade de código ao longo do desenvolvimento rápido de features:

- **Arquivos acima do limite de 300 linhas**: `hls-player.ts` (~320), `metadataStore.ts` (~310), `dispatcher.ts` (~330)
- **Uso extensivo de `any`**: Mais de 30 ocorrências em stores, strategies, renderers e UI components
- **`console.log` de debug**: Logs de depuração deixados em produção em ~6 arquivos
- **Funções com complexidade ciclomática > 10**: `handleBatchChange` (~34), `onDrop` no `TagDropStrategy` (~18)

Esses problemas violavam as [diretrizes frontend do projeto](../frontend-guidelines.md) e dificultavam a manutenção.

---

## Passo a Passo da Implementação

### Fase 1: Extração do HLS Player (3 arquivos)

**Problema:** `src/lib/hls-player.ts` continha uma classe (`HlsPlayerManager`), um hook SolidJS (`createHlsPlayer`), e funções utilitárias — tudo em ~320 linhas.

**Ações:**

1. **Extrair `HlsPlayerManager`** → novo arquivo `src/lib/hls-manager.ts`
   - Classe completa com gerenciamento de ciclo de vida do HLS
   - Refatoração de nomes: `src` → `sourceAccessor`, `cfg` → `hlsConfig`
   - Extração de `handleFatalError` como método privado (era inline)

2. **Extrair `createHlsPlayer`** → novo arquivo `src/lib/createHlsPlayer.ts`
   - Hook SolidJS com estado reativo
   - Extração de `attachHlsSource`/`attachNativeSource` como helpers
   - Parâmetro `src` renomeado para `sourceAccessor` (regra de naming)

3. **Slim down `hls-player.ts`** → apenas tipos, constantes e utilidades (~70 linhas)
   - Re-exports de `HlsPlayerManager` e `createHlsPlayer` para manter backward compatibility
   - Todos os importadores existentes continuam funcionando sem mudança

**Resultado:** 320 linhas → 70 + 124 + 107 = 3 arquivos bem focados.

---

### Fase 2: Redução de Complexidade do MetadataStore (2 arquivos)

**Problema:** `handleBatchChange` em `metadataStore.ts` tinha complexidade ciclomática ~34 — misturava lógica de estatísticas, notificações toast, e detecção de pastas desconhecidas.

**Ações:**

1. **Extrair lógica de estatísticas** → novo arquivo `src/core/store/statsHelpers.ts` (191 linhas)
   - `applyRemovals()` — recalcula stats após itens removidos
   - `applyAdditions()` — recalcula stats após itens adicionados
   - `applyUpdates()` — recalcula stats após itens atualizados
   - `computeStatsFromBatchChange()` — orquestrador que chama as 3 acima
   - Todas as funções são **puras** (recebem estado + payload, retornam novo estado)

2. **Extrair helpers no próprio metadataStore**
   - `showBatchChangeToasts(payload)` — lógica de notificação
   - `hasUnknownFolders(payload, locations)` — detecção de pastas não mapeadas

3. **Tipagem forte do payload**
   - `any` → `BatchChangePayload` (importado de `libraryStore`)
   - `any` → `SearchGroup | null` no `saveSmartFolder`

**Resultado:** Complexidade de `handleBatchChange`: 34 → ~8.

---

### Fase 3: Decomposição do Input Dispatcher (1 arquivo novo)

**Problema:** `src/core/input/dispatcher.ts` excedia 300 linhas e continha um sistema de eventos pub/sub inline.

**Ações:**

1. **Extrair CommandBus** → novo arquivo `src/core/input/commandBus.ts` (39 linhas)
   - `onCommand(name, handler)` — subscribe
   - `emitCommand(name, payload?)` — publish
   - `clearCommandHandlers()` — cleanup
   - Map de handlers tipado: `Map<string, CommandHandler[]>`

2. **Atualizar dispatcher.ts**
   - Importa e re-exporta de `commandBus.ts` para backward compatibility
   - Removido `(event as any).preventDefault()` → `'preventDefault' in event`

**Resultado:** 330 → ~295 linhas no dispatcher.

---

### Fase 4: Eliminação de `any` Types (10+ arquivos)

| Arquivo | Antes | Depois |
|---|---|---|
| `filterStore.ts` | `let searchDebounceTimer: any` | `ReturnType<typeof setTimeout> \| undefined` |
| `TagDropStrategy.ts` | 6× `(t: any)` em `.find()`, `.filter()`, `.sort()`, `.map()` | Importação do type `Tag` de `tags.ts`, tipagem explícita |
| `shortcutStore.ts` | `(t.meta as any)?.modifiers` | `token.meta?.modifiers` (meta já é `Record<string, unknown>`) |
| `AudioRenderer.tsx` | `(i: any) => i.id.toString()` | `libraryItem => libraryItem.id.toString()` |
| `VirtualListView.tsx` | `filters.setSortBy(key as any)` | `filters.setSortBy(key as SortField)` + import |
| `useVideoPlayer.ts` | `(container as any).webkitRequestFullscreen` | `'webkitRequestFullscreen' in container` + intersection type |
| `FontToolbar.tsx` | `value: any` | `value: FontSettings[keyof FontSettings]` |
| `AdvancedSearchModal.tsx` | `val: any` em `formatToISO`, `metadata: any` | `val: Date \| string`, metadata com tipo explícito `{ locations: ...; tags: ... }` |
| `GeneralPanel.tsx` | `setSetting('key', parseInt(val))` (number → string mismatch) | `setSetting('key', val)` / `setSetting('key', String(days))` |

---

### Fase 5: Remoção de `console.log` de Debug (4 arquivos)

| Arquivo | Log Removido |
|---|---|
| `FolderTreeSidebarPanel.tsx` | `console.log('Adding folder:', selected)` |
| `ReferenceImage.tsx` | `console.log(\`Thumbnail ready for image ID: ...\`)` |
| `ReferenceImage.tsx` | `console.log(\`Requesting thumbnail regeneration...\`)` |
| `TagDropStrategy.ts` | `console.log(\`Assigning images...\`)` e `console.log(\`Moving tag...\`)` |

**Nota:** `console.error` e `console.warn` foram mantidos — são logs legítimos de erro/aviso.

---

### Fase 6: Rewrite Completo do TagDropStrategy (1 arquivo)

**Problema:** O arquivo original tinha:
- `eslint-disable` no topo (desabilitava 4 regras)
- 6 usos de `any` em callbacks de array
- 2 `console.log` de debug
- Complexidade ciclomática ~18 no `onDrop`

**Ações:**

1. Removido o `eslint-disable` completamente
2. Importado o type `Tag` de `tags.ts`
3. Extração de 5 funções auxiliares:
   - `handleImageDrop()` — lógica de atribuição de imagens a tag
   - `handleTagDrop()` — lógica de reordenação/nesting de tags
   - `resolveNewParentId()` — determina novo parent baseado na posição de drop
   - `buildReorderedSiblings()` — monta lista ordenada com tag inserida na posição correta
   - `createReorderUpdates()` — gera chamadas de `tagService.updateTag()` para todos os siblings afetados
4. Método `onDrop` reduzido a ~15 linhas de orquestração

**Resultado:** Complexidade ~18 → ~4.

---

## Obstáculos Encontrados

### 1. Duplicação de código no replacement parcial

Ao substituir apenas a primeira parte da função `computeDisplayValue` em `AdvancedSearchModal.tsx`, o código restante (que também estava no escopo de substituição) ficou duplicado como código órfão fora de qualquer função. Isso causou erros de parsing.

**Solução:** Identificar e deletar manualmente as linhas duplicadas restantes com uma segunda operação de edição.

**Lição:** Ao refatorar funções longas com replacement parcial, é mais seguro substituir a função inteira ou usar replace com boundaries bem definidos.

### 2. Union type `TagDragPayload | Record<string, unknown>` no DnD

O tipo `DragItem.payload` é uma union de `TagDragPayload | Record<string, unknown>`. Quando `item.type === 'IMAGE'`, o payload tem `.ids`, mas TypeScript não consegue narrowing automático baseado no `type` field (não é discriminated union).

**Solução:** Cast intermediário: `const imagePayload = item.payload as Record<string, unknown>; const imageIds = imagePayload.ids as number[];`

**Melhoria futura:** Converter `DragItem` em discriminated union:
```typescript
type DragItem = 
  | { type: 'IMAGE'; payload: { ids: number[] } }
  | { type: 'TAG'; payload: TagDragPayload };
```

### 3. `setSetting` signature mismatch

`tauriService.setSetting()` foi tipada para aceitar `string` como valor, mas `GeneralPanel.tsx` passava `parseInt(val)` (number). Isso era mascarado pelo `any` implícito anterior e só surgiu após a eliminação dos `any`.

**Solução:** Passar o valor já como string (`val` direto ou `String(days)`).

### 4. `val / m` arithmetic em `SearchCriterion.value`

Após mudar `SearchCriterion.value` de `any` para `string | number | boolean | null`, operações aritméticas como `val / m` pararam de compilar, pois `string` e `boolean` não suportam divisão.

**Solução:** Encapsular com `Number(val)` para garantir conversão numérica antes da aritmética.

---

## Arquivos Modificados

| Arquivo | Tipo | Mudança |
|---|---|---|
| `src/lib/hls-manager.ts` | **Novo** | `HlsPlayerManager` extraído |
| `src/lib/createHlsPlayer.ts` | **Novo** | Hook `createHlsPlayer` extraído |
| `src/lib/hls-player.ts` | Refatorado | Slimmed → tipos + utils + re-exports |
| `src/core/store/statsHelpers.ts` | **Novo** | Funções puras de computação de stats |
| `src/core/input/commandBus.ts` | **Novo** | Sistema pub/sub de comandos |
| `src/core/store/metadataStore.ts` | Refatorado | Tipagem + extração de helpers |
| `src/core/store/filterStore.ts` | Fixado | Timer type `any` → `ReturnType` |
| `src/core/input/dispatcher.ts` | Refatorado | Importa de commandBus, remove `any` |
| `src/core/input/store/shortcutStore.ts` | Fixado | `(t.meta as any)` → `token.meta` |
| `src/core/dnd/strategies/TagDropStrategy.ts` | **Rewrite** | Remove `any`, `console.log`, eslint-disable |
| `src/components/features/library/FolderTreeSidebarPanel.tsx` | Fixado | Remove `console.log` |
| `src/components/features/viewport/ReferenceImage.tsx` | Fixado | Remove 2× `console.log` |
| `src/components/features/viewport/VirtualListView.tsx` | Fixado | `as any` → `as SortField` |
| `src/components/features/itemview/renderers/audio/AudioRenderer.tsx` | Fixado | `(i: any)` → tipo correto |
| `src/components/features/itemview/renderers/font/FontToolbar.tsx` | Fixado | `value: any` → `FontSettings[keyof FontSettings]` |
| `src/components/features/search/AdvancedSearchModal.tsx` | Fixado | 3 `any` → tipos concretos |
| `src/components/features/settings/GeneralPanel.tsx` | Fixado | `setSetting` recebe `string` |
| `src/components/ui/VideoPlayer/useVideoPlayer.ts` | Fixado | webkit fullscreen `any` → type guards |

---

## Verificação

```
npx tsc --noEmit  →  Found 0 errors  ✅
npm run tauri dev →  Compilação bem-sucedida  ✅
```

---

## Melhorias Futuras

### Alta Prioridade

1. **`useVideoPlayer.ts` — 373 linhas (acima do limite 300)**
   Extrair lógica de fullscreen, volume, e HLS attachment para hooks separados:
   - `useFullscreen.ts` (~30 linhas)
   - `usePlayerVolume.ts` (~20 linhas) 
   - `useHlsAttachment.ts` (~30 linhas)

2. **`AdvancedSearchModal.tsx` — 1191 linhas (massivamente acima do limite)**
   Este componente é o maior do projeto. Recomendações:
   - Extrair `CriteriaBuilder` como sub-componente
   - Extrair `QueryEditor` como sub-componente
   - Extrair `CriterionItem` com lógica de edição inline
   - Mover helpers (`formatToISO`, `formatToDisplay`, `computeDisplayValue`) para arquivo separado
   - Restam ~6 usos de `any` nos signals de estado (`currentValue`, `editingValue`)

3. **`DragItem` como discriminated union**
   Converter de `payload: TagDragPayload | Record<string, unknown>` para union com discriminante:
   ```typescript
   type DragItem = 
     | { type: 'IMAGE'; payload: { ids: number[] } }
     | { type: 'TAG'; payload: TagDragPayload };
   ```

### Média Prioridade

4. **`DropdownMenu.tsx` — 10× `as any` casts**
   O menu suporta múltiplos tipos de items (normal, checkbox, radio, submenu, label, separator). Precisa de uma discriminated union no `MenuItem` type para eliminar os casts.

5. **`Input.tsx` — 3× `as any` em event handlers**
   `(others.onFocus as any)(e)` — problema com tipagem de spread props no SolidJS. Investigar se `splitProps` resolve.

6. **`TreeView.tsx` — 2× `as any` no DnD**
   Similar ao `TagDropStrategy` — precisa de tipagem mais forte nos drag payloads.

7. **`Table.tsx` — 2× `as any`**
   `item[keyField()] as any` e `item[col.accessorKey as keyof T] as any` — requer constraintg generics mais fortes (`T extends Record<string, unknown>`).

### Baixa Prioridade

8. **`DesignSystemGuide.tsx` — 1× `as any`**
   Componente de referência/documentação, baixo impacto em produção.

9. **`ContextMenu.tsx` — 1× `as any`**
   Similar ao DropdownMenu, precisa de discriminated union nos item types.
