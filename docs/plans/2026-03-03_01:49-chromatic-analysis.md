# Análise Cromática Nativa — Extração de Paleta e Busca por Cor

**Status:** ✅ Implementação Concluída (pendente: testes manuais + sqlx prepare)  
**Data de Criação:** 2026-03-03  
**Brainstorm:** [brainstorm_chromatic_analysis.md](file:///Users/marcusmaia/.gemini/antigravity/brain/0142aeb8-b002-46d1-9a7d-66acc68ceea5/brainstorm_chromatic_analysis.md)

---

## Sumário

Sistema completo de extração de paleta de cores de imagens, persistência no banco de dados, exibição no Inspector e busca por proximidade de cor no Advanced Search.

**Princípio Core (alinhado com Eagle):** Analisar os **códigos de cor reais dos pixels** das imagens — não tags pré-definidas — para obter resultados de busca extremamente precisos.

---

## Decisões Técnicas Confirmadas

| Decisão | Escolha | Detalhes |
|---------|---------|----------|
| Algoritmo | K-Means CIE-LAB (`kmeans-colors` crate) | k=16, input = thumbnail gerado |
| Schema DB | Tabela `asset_colors` + coluna `dominant_color` em `assets` | Híbrido: read rápido + busca precisa |
| Timing | Pipeline de thumbnailing existente | Apenas `MediaType::Image` |
| Busca | Distância Euclidiana CIE-76 via SQL | Slider accuracy → threshold ΔE |
| Harmonia | Classificação por hue dos clusters agglomerativos 3D | 13 tipos: mono, comp, analog, triad, split, tetrad, square, dyad, accented, achro, neutral, poly, n/a |
| Distribuição | Agglomerative Clustering em espaço cilíndrico HSL 3D | 3-5 grupos finais, regra 60-30-10 |
| Re-processamento | Sim, toda biblioteca ou asset individual | Comando dedicado |

---

## Plano de Implementação

### Fase 1: Backend — Schema e Migration

**Objetivo:** Criar a estrutura de banco de dados para armazenar paletas de cores.

**Arquivos afetados:**
- `src-tauri/migrations/20260303000000_add_color_analysis.sql` *(novo)*
- `src-tauri/src/db/models.rs` *(edição)*
- `src-tauri/src/db/mod.rs` *(edição)*

#### Step 1.1: Criar Migration SQL
- [x] **Status:** ✅ Concluído

Criar `src-tauri/migrations/20260303000000_add_color_analysis.sql`:

```sql
-- Color Analysis: stores extracted color palette for image assets
ALTER TABLE assets ADD COLUMN dominant_color TEXT;

CREATE TABLE IF NOT EXISTS asset_colors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    hex_color TEXT NOT NULL,
    lab_lightness REAL NOT NULL,
    lab_green_red REAL NOT NULL,
    lab_blue_yellow REAL NOT NULL,
    percentage REAL NOT NULL,
    rank INTEGER NOT NULL,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_asset_colors_asset ON asset_colors(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_colors_lab ON asset_colors(lab_lightness, lab_green_red, lab_blue_yellow);
```

#### Step 1.2: Atualizar Models
- [x] **Status:** ✅ Concluído

Adicionar `dominant_color: Option<String>` ao struct `AssetMetadata` em `db/models.rs`.

Criar struct `AssetColor` em `db/models.rs`:
```rust
/// A single extracted color from an asset's palette.
pub struct AssetColor {
    pub id: i64,
    pub asset_id: i64,
    pub hex_color: String,
    pub lab_lightness: f64,
    pub lab_green_red: f64,
    pub lab_blue_yellow: f64,
    pub percentage: f64,
    pub rank: i32,
}
```

#### Step 1.3: Criar Módulo DB `colors.rs`
- [x] **Status:** ✅ Concluído

Criar `src-tauri/src/db/colors.rs` com funções:
- `insert_asset_colors(&self, asset_id: i64, colors: &[AssetColor])` — insere cores em batch (transaction)
- `get_asset_colors(&self, asset_id: i64) -> Vec<AssetColor>` — retorna paleta de um asset
- `delete_asset_colors(&self, asset_id: i64)` — limpa cores de um asset (para re-extração)
- `update_dominant_color(&self, asset_id: i64, hex: &str)` — atualiza coluna `dominant_color`
- `search_assets_by_color(&self, lab_l, lab_a, lab_b, threshold) -> Vec<i64>` — busca por proximidade

Registrar em `db/mod.rs` com `pub mod colors;`.

---

### Fase 2: Backend — Algoritmo de Extração

**Objetivo:** Implementar o k-means no espaço CIE-LAB para extrair paletas de thumbnails.

**Arquivos afetados:**
- `src-tauri/Cargo.toml` *(edição — adicionar `kmeans-colors` e `palette`)*
- `src-tauri/src/thumbnails/color_analysis.rs` *(novo)*
- `src-tauri/src/thumbnails/mod.rs` *(edição)*

#### Step 2.1: Adicionar Dependências
- [x] **Status:** ✅ Concluído (crate name: `kmeans_colors` com underscore)

Adicionar ao `Cargo.toml`:
```toml
kmeans-colors = "0.6"
palette = "0.7"
```

Crate `kmeans-colors` faz k-means otimizado no espaço LAB.  
Crate `palette` faz conversão RGB↔LAB de forma precisa.

#### Step 2.2: Implementar `color_analysis.rs`
- [x] **Status:** ✅ Concluído

Criar `src-tauri/src/thumbnails/color_analysis.rs` com:

```rust
// Pseudocódigo da API pública
pub struct ExtractedColor {
    pub hex: String,
    pub lab_l: f64,
    pub lab_a: f64,
    pub lab_b: f64,
    pub percentage: f64,
}

pub fn extract_color_palette(
    thumbnail_path: &Path,
    cluster_count: usize, // default 16
) -> Result<Vec<ExtractedColor>, Box<dyn Error>>
```

**Fluxo interno:**
1. Carregar thumbnail via `image::open()` (já ~256px, ideal)
2. Converter pixels RGBA para `palette::Lab`
3. Rodar `kmeans_colors::get_kmeans_hamerly()` com k=16, max_iter=20, seed fixo
4. Coletar centroides + pesos
5. Converter centroides LAB→RGB→Hex
6. Ordenar por percentage descendente
7. Retornar `Vec<ExtractedColor>`

**Atenção:**
- Ignorar pixels com alpha < 128 (transparência)
- Usar `seed = 42` para reprodutibilidade
- O thumbnail já é resize, então k-means roda em ~65k pixels max

#### Step 2.3: Registrar Módulo
- [x] **Status:** ✅ Concluído

Adicionar `pub mod color_analysis;` em `thumbnails/mod.rs`.

---

### Fase 3: Backend — Integração na Pipeline de Thumbnails

**Objetivo:** Executar extração de cores após geração bem-sucedida de thumbnail para assets do tipo `MediaType::Image`.

**Arquivos afetados:**
- `src-tauri/src/thumbnails/worker.rs` *(edição)*
- `src-tauri/src/formats/types.rs` *(referência — `MediaType::Image`)*

#### Step 3.1: Integrar no Worker Loop
- [x] **Status:** ✅ Concluído

No `worker_loop()`, após o thumbnail ser gerado com sucesso e antes do `db.update_thumbnail_path()`:

1. Verificar se o asset é `MediaType::Image` — consultar via `FileFormat::detect()` ou checar media_type do DB
2. Se sim, chamar `color_analysis::extract_color_palette(&thumb_path, 16)`
3. Se extração successful:
   - Chamar `db.delete_asset_colors(id)` (para suportar re-extração)
   - Chamar `db.insert_asset_colors(id, &colors)`
   - Chamar `db.update_dominant_color(id, &colors[0].hex)` (top 1)
4. Se falhar: logar warning e continuar (NÃO bloquear thumbnail)

**Ponto crítico:** A extração de cor **NÃO deve** falhar o thumbnail. Erros devem ser logados mas tolerados (`tracing::warn!`).

---

### Fase 4: Backend — Busca por Cor e Comandos Tauri

**Objetivo:** Expor endpoints para busca por cor e re-extração.

**Arquivos afetados:**
- `src-tauri/src/db/search.rs` *(edição)*
- `src-tauri/src/library/commands/` *(edição ou novo)*
- `src-tauri/permissions/main.toml` *(edição)*
- `src-tauri/capabilities/default.json` *(edição)*

#### Step 4.1: Adicionar Critério `color` ao Search Builder
- [x] **Status:** ✅ Concluído

Adicionar novo case em `build_criterion_clause()` para `key = "color"`:

```rust
"color" => {
    // value = [hex_color, threshold_delta_e]
    // Converter hex para LAB no backend
    // Buscar em asset_colors por distância euclidiana
    // WHERE asset_id IN (SELECT asset_id FROM asset_colors WHERE ΔE < threshold)
}
```

O valor virá como JSON `{ "hex": "#FF5733", "threshold": 25 }` do frontend.

#### Step 4.2: Criar Comandos Tauri
- [x] **Status:** ✅ Concluído

Criar comandos:
- `get_asset_colors(asset_id: i64) -> Vec<AssetColor>` — retorna paleta
- `reextract_colors(asset_id: i64)` — re-extrai cores de um asset específico
- `reextract_all_colors()` — re-extrai cores de toda a biblioteca

Seguir o pattern de thin commands (guideline: `backend-rust.md`).

#### Step 4.3: Atualizar Permissions
- [x] **Status:** ✅ Concluído

Registrar permissões em `permissions/main.toml` e `capabilities/default.json`.

#### Step 4.4: Executar `cargo sqlx prepare`
- [ ] **Status:** ⏳ Pendente (executar após primeira build completa com migration aplicada)

Rodar `cargo sqlx prepare` dentro de `src-tauri/` para atualizar o cache `.sqlx/` para CI offline.

---

### Fase 5: Frontend — Componentes do Inspector

**Objetivo:** Exibir paleta de cores no `ImageInspector` com 3 seções: harmonia, distribuição e swatches.

**Arquivos afetados:**
- `src/components/features/inspector/image/ImageInspector.tsx` *(edição)*
- `src/components/features/inspector/image/ColorPaletteSection.tsx` *(novo)*
- `src/components/features/inspector/image/ColorHarmonyBadge.tsx` *(novo)*
- `src/components/features/inspector/image/ColorDistribution.tsx` *(novo)*
- `src/components/features/inspector/image/ColorSwatchGrid.tsx` *(novo)*
- `src/components/features/inspector/image/color-palette.css` *(novo)*
- `src/components/features/inspector/image/colorHarmonyUtils.ts` *(novo — harmony types, display map, classificação)*
- `src/components/features/inspector/image/colorClusteringUtils.ts` *(novo — tipos, HSL, 3D, agglomerative clustering)*

#### Step 5.1: Utilitário de Harmonia
- [x] **Status:** ✅ Concluído (refinado 3x — última: algoritmo compartilhado + 5 novos tipos)

Arquivos (split por `max-lines: 300`):
- `colorClusteringUtils.ts` *(novo)* — tipos, HSL, 3D mapping, agglomerative clustering
- `colorHarmonyUtils.ts` *(refatorado)* — tipos de harmonia, display map, classificação, re-exports

Função `detectColorHarmony(clusters: ColorCluster[]): HarmonyType`:
- Recebe clusters pré-computados do agglomerative grouping (mesmos da distribuição)
- Extrai hue de cada centroide via `atan2(y, x)` e saturação via `sqrt(x² + y²)`
- Separa clusters em: achromatic (S < 0.03), neutral (S < 0.08), chromatic (S ≥ 0.08)
- Classifica relações angulares entre hues cromáticos

**13 Tipos de Harmonia Suportados:**

| Tipo | Clusters | Critério Angular |
|------|----------|-------------------|
| `monochromatic` | 1 cromático | Hue único |
| `complementary` | 2 | ΔH 150°–210° |
| `dyadic` | 2 | ΔH 45°–80° |
| `analogous` | 2–3 | ΔH ≤ 45° (2) ou todos ≤ 60° (3) |
| `triadic` | 3 | Todos ΔH 90°–150° |
| `split_complementary` | 3 | Menor ≤ 60°, maior ≥ 140° |
| `accented_analogous` | 3 | Par ≤ 45° + acento ~180° do midpoint |
| `square` | 4 | 4 deltas ~90° + 2 deltas ~180° |
| `tetradic` | 4+ | ≥60% deltas ~90° ou ~180° |
| `polychromatic` | 4+ | Fallback: muitos hues distintos |
| `achromatic` | qualquer | TODOS clusters S < 0.03 |
| `neutral` | qualquer | Nenhum cluster S ≥ 0.08 |
| `not_identified` | 2–3 | Não encaixa em padrão |

**Refinamentos aplicados (cronológico):**
1. Versão inicial analisava apenas top 5 cores → classificação incorreta. Corrigido para todas com percentage ≥ 1%.
2. Lógica unificada com distribuição: ambos usam `agglomerativeGrouping()`. Harmony recebe clusters pré-computados.
3. Arquivo `colorHarmonyUtils.ts` dividido em 2 para compliance com `max-lines: 300`.
4. 5 novos tipos adicionados: `square`, `dyadic`, `accented_analogous`, `achromatic`, `polychromatic`.
5. Separadores visuais (`// ===`) removidos conforme guidelines de documentação.

#### Step 5.2: Componente `ColorHarmonyBadge`
- [x] **Status:** ✅ Concluído

Badge/chip visual mostrando o tipo de harmonia detectada.
- Mapeamento de ícone e label para cada tipo
- Tooltip com breve explicação (ex: "Colors are opposite on the color wheel")

#### Step 5.3: Componente `ColorDistribution`
- [x] **Status:** ✅ Concluído (refatorado 2x após testes manuais)

Barra horizontal stacked mostrando distribuição proporcional de famílias de cor.

**Evolução da implementação:**

1. **v1 — Top N direto:** Mostrava as top 5 cores brutas do k-means. Problema: as % não somavam 100%, deixando gap preto na barra; cores similares apareciam separadas.

2. **v2 — Agrupamento por Hue (greedy):** Agrupava todas as 16 cores por proximidade de matiz (1D) com busca gulosa. Problema: considerava apenas hue, fundindo cores visualmente distintas (ex: vermelho escuro + vermelho claro); busca gulosa impedia agrupamento globalmente ótimo.

3. **v3 — Agglomerative Clustering 3D (implementação final):** Refatoração estrutural com 3 etapas:

   **Etapa 1 — Mapeamento 3D (Cylindrical HSL → Cartesian):**
   ```
   x = S · cos(H_rad)
   y = S · sin(H_rad)
   z = L
   ```
   Distância entre cores = distância euclidiana 3D, refletindo diferença visual real.

   **Etapa 2 — Agglomerative Hierarchical Clustering:**
   - Cada cor inicia como seu próprio cluster
   - A cada iteração, encontra o par de clusters **globalmente** mais próximo (O(n²))
   - Mescla com centroide ponderado por percentagem
   - Não é greedy: analisa todas as distâncias antes de decidir

   **Etapa 3 — Condição de Parada (3-5 grupos):**
   - Continua mesclando até ter 3–5 clusters
   - Se a distância do par mais próximo > `MERGE_DISTANCE_THRESHOLD` (0.35), para mesmo com >5 clusters
   - Cor representativa = maior percentagem individual dentro do cluster

   **Resultado:** Barra preenche 100%, segue padrão 60-30-10, separa corretamente cores como vermelho escuro vs. vermelho claro.

#### Step 5.4: Componente `ColorSwatchGrid`
- [x] **Status:** ✅ Concluído

Grid de retângulos coloridos (até 16 swatches):
- Cada swatch exibe a cor com borda sutil (mínimo 32px)
- Ao clicar: `navigator.clipboard.writeText(hexColor)`
- Feedback visual com ícone ✓ (checkmark) por ~1.5s — substituiu texto "Copied!" que era cortado em swatches pequenos
- Acessibilidade: `aria-label` com hex para cada swatch, suporte a teclado (Enter/Space)

#### Step 5.5: Container `ColorPaletteSection`
- [x] **Status:** ✅ Concluído

Componente wrapper que:
1. Usa `createResource` com `invoke('get_asset_colors', { assetId })` direto (sem passar por tauriService)
2. Passa as cores para os 3 sub-componentes
3. Calcula harmonia no frontend via `detectColorHarmony()`
4. Encapsulado em `<AccordionItem>`

#### Step 5.6: Integrar no `ImageInspector`
- [x] **Status:** ✅ Concluído

Adicionar `<ColorPaletteSection item={props.item} />` como novo `Accordion.Item` no `ImageInspector.tsx`, após `<AdvancedMetadata>`.

---

### Fase 6: Frontend — Busca por Cor no Advanced Search

**Objetivo:** Adicionar campo de busca por cor com seletor de cor e slider de proximidade.

**Arquivos afetados:**
- `src/components/features/search/fields/ColorCriterionField.tsx` *(novo)*
- `src/components/features/search/fields/index.ts` *(edição)*
- `src/core/store/filter/constants.ts` *(edição)*
- `src/core/store/filter/schemas.ts` *(edição — ampliação do tipo `value`)*
- `src/core/store/filter/logic/handlers.ts` *(edição — novo `colorLogic`)*
- `src/components/features/search/useAdvancedSearch.ts` *(edição — `handleStartEdit` para color)*
- `src-tauri/src/db/search.rs` *(edição — Fase 4 já cobre)*

#### Step 6.1: Criar `ColorCriterionField.tsx`
- [x] **Status:** ✅ Concluído

Componente que renderiza:
1. `<ColorInput>` (componente existente) para seleção de cor
2. `<Slider>` (componente existente) para ajustar proximidade/accuracy (0-100)
   - 0 = correspondência exata (ΔE < 2.3)
   - 50 = similar (ΔE ~25)
   - 100 = família ampla (ΔE ~50)
3. Label indicando a intensidade selecionada ("Exact", "Similar", "Broad")

Exportar `colorHandler: SearchFieldHandler` com:
- `validate`: verificar que hex é válido
- `process`: enviar `{ hex, threshold: mappedDeltaE }` como value
- `formatDisplay`: mostrar swatch de cor + nível de accuracy

**Formato de dados interno vs. processado:**
- **State interno:** JSON string `'{"hex":"#FF0000","proximity":50}'` — proximity é slider % (0-100)
- **Valor processado:** Objeto `{ hex: "#FF0000", threshold: 25.6 }` — threshold é ΔE real
- Conversão: `threshold = 2.3 + (proximity / 100) × (50 − 2.3)`
- Reverso (edit mode): `proximity = ((threshold − 2.3) / (50 − 2.3)) × 100`

**Funções utilitárias no componente:**
- `parseColorValue(raw)` — parser unificado que aceita tanto JSON string quanto objeto processado
- `extractFromObject(obj)` — helper para extrair hex/proximity de um objeto (com ou sem threshold)
- `sliderPercentageToDeltaE(%)` / `deltaEToSliderPercentage(ΔE)` — conversão bidirecional

#### Step 6.2: Registrar no Handler Registry
- [x] **Status:** ✅ Concluído

Em `fields/index.ts`, adicionar:
```typescript
import { colorHandler } from './ColorCriterionField';
// ...
color: colorHandler,
```

#### Step 6.3: Adicionar à Lista de Search Fields
- [x] **Status:** ✅ Concluído

Em `core/store/filter/constants.ts`, adicionar:
```typescript
{ value: 'color', label: 'Color', type: 'color' },
```

E adicionar operadores para tipo `color`:
```typescript
color: [
    { value: 'similar', label: 'Similar to' },
    { value: 'exact', label: 'Exact match' },
],
```

#### Step 6.4: Adicionar `colorLogic` ao Store Registry
- [x] **Status:** ✅ Concluído (necessário para a pipeline funcionar)

O `criterionLogicRegistry` em `src/core/store/filter/logic/handlers.ts` é a camada do store que processa os valores dos critérios. Sem um entry `color`, o store usava `textLogic` como fallback, que passava o valor como string inalterada.

Adicionado `colorLogic: SearchFieldLogic` com:
- `validate`: verifica hex válido via regex
- `process`: converte JSON string `{hex, proximity}` → objeto `{hex, threshold}` (ΔE calculado)
- `formatDisplay`: exibe `#hex (Exact|Similar|Broad)` a partir do threshold armazenado

#### Step 6.5: Ampliar `SearchCriterionSchema` para aceitar objetos
- [x] **Status:** ✅ Concluído

O Zod schema e a interface `SearchCriterion` só aceitavam `string | number | boolean | null | array`. O valor processado do color é um objeto `{ hex, threshold }`, que era rejeitado pela validação.

Alterações em `src/core/store/filter/schemas.ts`:
- Schema: adicionado `z.record(z.string(), z.unknown())` ao union de `value`
- Interface: adicionado `Record<string, unknown>` ao tipo `value`

#### Step 6.6: Corrigir Edit Mode para Color
- [x] **Status:** ✅ Concluído

Em `useAdvancedSearch.ts`, `handleStartEdit` não sabia converter o valor processado `{hex, threshold}` de volta para o formato interno `{hex, proximity}` que o `ColorCriterionField` espera.

Adicionado tratamento específico para `criterionItem.key === 'color'`:
- Extrai threshold do objeto armazenado
- Calcula proximity via `((threshold - 2.3) / (50 - 2.3)) × 100`
- Serializa como JSON string para o componente

---

### Fase 7: Testes, Compilação e Validação

#### Step 7.1: Verificar Compilação Backend
- [x] **Status:** ✅ Concluído (`cargo check` passou com sucesso)

```bash
cd src-tauri && cargo check && cargo clippy -- -D warnings
```

#### Step 7.2: Verificar Frontend
- [x] **Status:** ✅ Concluído (`npx tsc --noEmit` e `npx eslint` passaram sem erros)

```bash
npm run lint && npx tsc --noEmit
```

#### Step 7.3: Testes Manuais
- [ ] **Status:** ⏳ Pendente

1. Iniciar app e importar pasta com imagens
2. Aguardar thumbnails serem gerados
3. Verificar no Inspector que as cores aparecem
4. Clicar em swatch e verificar cópia para clipboard
5. Testar busca por cor no Advanced Search com diferentes thresholds
6. Testar re-extração via comando

#### Step 7.4: Executar `cargo sqlx prepare`
- [ ] **Status:** ⏳ Pendente

---

## Melhorias Futuras Documentadas

> [!NOTE]
> Estas são melhorias identificadas durante o brainstorm que podem ser implementadas futuramente sem impactar a v1.

1. **CIEDE2000**: Substituir distância CIE-76 (euclidiana) por CIEDE2000 para precisão perceptual máxima em diferenças sutis de cor. Medium effort, high impact para matching refinado.

2. **Grid Bins (Spatial Index)**: Se a performance de busca por cor degradar com >100k assets, implementar binning LAB quantizado para range queries indexadas (Option C do brainstorm). 

3. **Paleta de Vídeos**: Extrair paleta do frame principal de thumbnails de vídeo. Requer decisão sobre qual frame representa melhor o vídeo.

4. **Busca Multi-Cor**: Permitir buscar por combinações de cores (ex: "imagens com vermelho E azul em proporções similares").

5. **Export de Paleta**: Exportar paleta como `.ase` (Adobe Swatch Exchange), `.gpl` (GIMP), ou `.json` para uso em ferramentas de design.

6. **Histograma de Cor**: Visualização alternativa como histograma de matiz/saturação além dos swatches.

7. **Dominant Color Badge**: Exibir small swatch da `dominant_color` diretamente no `AssetCard` do viewport (sem abrir Inspector).

---

## Obstáculos e Notas

> [!WARNING]
> Obstáculos encontrados durante a implementação:

1. **Crate name:** O crate `kmeans-colors` no crates.io usa underscores (`kmeans_colors`), não hyphens.
2. **sqlx compile-time macros:** `sqlx::query!` e `sqlx::query_as!` validam contra o DB em tempo de compilação. Como a migration ainda não foi aplicada, `db/colors.rs` usa runtime `sqlx::query` (sem macro) para evitar falhas. Após a migration ser aplicada, pode-se converter para macros se desejado.
3. **Send + Sync:** O error type `Box<dyn Error>` precisa ser `Box<dyn Error + Send + Sync>` para satisfazer o bound `Send` dos comandos Tauri async.
4. **thumb_dir_clone moved:** O `spawn_blocking` move o clone, necessitando um segundo clone (`thumb_dir_for_colors`) para o bloco de extração de cores.
5. **Harmonia monochromatic falsa:** Top 5 cores do k-means podem ser variações do mesmo matiz dominante quando k=16. Resolvido analisando todas as cores com percentage >= 1%.
6. **Distribuição com gap preto:** Percentagens das top N cores não somavam 100%. Resolvido com normalização (v2) e depois com agglomerative clustering (v3) que garante cobertura completa.
7. **Swatch "Copied!" cortado:** Texto "Copied!" com 9px não cabia em swatches de 28px com `overflow: hidden`. Resolvido substituindo por checkmark ✓ (14px) e aumentando swatch mínimo para 32px.
8. **TypeScript type mismatch em ColorCriterionField:** `SearchValue = string | number | Date | null` não pode ser castado diretamente para `{ hex, proximity }`. Resolvido tratando apenas `typeof === 'string'` via `JSON.parse` com fallbacks seguros.
9. **max-lines excedido:** Após adição de 5 novos tipos de harmonia, `colorHarmonyUtils.ts` excedeu 300 linhas. Resolvido dividindo em `colorClusteringUtils.ts` (tipos + HSL + clustering) e `colorHarmonyUtils.ts` (harmonia + re-exports).
10. **Separadores visuais proibidos:** Comentários `// ===` violavam guideline de documentação. Removidos após revisão.
11. **Busca por cor retornava todos os assets (3 bugs encadeados):**
    - **Bug A — `criterionLogicRegistry` sem `color`:** O store caía no fallback `textLogic.process()`, que retornava o valor como string inalterada. Backend recebia `c.value` como string JSON, não objeto, fazendo `c.value.get("hex")` retornar `None` → fallback `#000000` → match em tudo.
    - **Bug B — `SearchCriterionSchema` rejeitava objetos:** O Zod schema só aceitava `string | number | boolean | null | array`. Objeto `{hex, threshold}` era silenciosamente rejeitado por `setAdvancedSearch()`.
    - **Bug C — `colorHandler.process()` fazia `JSON.stringify()`:** Mesmo se o store usasse o handler correto, o `finalValue` seria uma string dupla-serializada.
    - **Resolução:** Adicionado `colorLogic` ao registry + ampliado schema + removido stringify.
12. **Edit mode não restaurava valores do color:** `handleStartEdit` recebe o valor processado `{hex, threshold}` mas o componente espera `{hex, proximity}` como JSON string. Sem conversão reversa, o campo abria com `#000000` e proximity default. Resolvido com `deltaEToSliderPercentage()` e serialização explícita em `handleStartEdit`.
13. **`formatDisplay` acessava campo inexistente:** Após processamento, o valor armazenado tem `threshold`, não `proximity`. `formatDisplay` tentava ler `parsed.proximity` → `undefined` → label sempre "Broad". Resolvido calculando proximity reverso a partir de threshold.

---

## Referências

- [Roadmap L78-L80](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/docs/report/2026-02-20_roadmap.md#L78-L80)
- [Plano de Ação L33-L36](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/docs/report/2026-02-20_plano_acao_pendencias.md#L33-L36)
- [Backend Guidelines](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/docs/guidelines/backend-rust.md)
- [Frontend Guidelines](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/docs/guidelines/frontend-solid.md)
- [Core Architecture](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/docs/guidelines/core-architecture.md)
- [Documentation Standards](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/docs/guidelines/documentation.md)
- Crate: [kmeans-colors](https://crates.io/crates/kmeans-colors) v0.6
- Crate: [palette](https://crates.io/crates/palette) v0.7
- Concorrente referência: Eagle (color filtering by actual pixel analysis)
