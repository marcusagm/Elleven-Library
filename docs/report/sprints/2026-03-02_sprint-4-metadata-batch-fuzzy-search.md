# Sprint 4: Advanced Metadata Batch Operations & Fuzzy Search

**Data:** 2026-03-02
**Status:** Em andamento
**Data e hora da conclusão:** -

## 📌 Objetivo
Avançar com recursos definitivos para manipulação em massa e buscas de tolerância alta ("Search & Tagging") preenchendo os gaps de Usabilidade Profissional da Fase 2. Isso assegura que governar o acervo com comandos complexos para múltiplos objetos ou descrições imprecisas seja fluido para o *Power-User*.

## 🛠 Tarefas de Implementação

### [✓] 1. Masterização no Batch Tagging (`MultiInspector.tsx`)
- **Escopo (Concluído):** Interações dinâmicas sobre metadados operando confiavelmente sob milhares de fotos. Foi adicionado feedback real-time ligando a reatividade visual (`tagUpdateVersion`) aos hooks (`useMetadata`) e `EventBus` para drag-and-drops com zero latência perceptível.
- **Ações (Frontend & Backend):**
  - **Frontend:** Desenhar `MultiInspector.tsx` através de Atomic Composition ou Compound Components para exibir estados "Indeterminados" (elementos onde nem todos da seleção possuem determinada tag). As actions subjacentes jamais operarão em map-loop sobre item único; chamarão abstrações unificadas no hook (`metadataActions.updateAssetsTags`).
  - **Backend:** Acionamento unificado do `metadata_update_batch` suportando deltas lógicos (ApplyTag, RemoveTag) otimizados em operações batch e `tagsService`.
- **Validação:** Seleções imensas e arrastáveis (>10.000 imagens/vetoriais) podem ser tageadas sob a margem temporal rápida. Ausência terminante de manipulação de string manual no TS e obediência à tipagem.

### 2. Busca Tolerante Sintática & "Fuzzy Search"
- **Escopo:** Substituir as falhas de pesquisa cruas que dependam exclusivamente de sintaxe ortográfica perfeita, entregando algoritmos leves e inteligentes no catálogo (SQLite local).
- **Ações:**
  - Instanciar a extensão dedicada de distanciamento no SQLite (como `spellfix1`, `FTS5`, algorítmos atados de `Levenshtein` ou `Trigram` nativos). Manter implantação leve minimizando complexidade adicional pesada no OS host.
  - A lógica unifica-se à base de `QueryBuilder` existente (`src/db/search.rs`), em que na ativação nativa do frontend o usuário passa as diretrizes como "Buscar similaridades" com matching de precisão ou erro adaptativo.
  - O frontend refletirá os matches com score de semelhança (fuzzy logic) para garantir relevância no output da VirtualList e AdvancedSearchModal, descartando "ruídos".
- **Validação:** Usuários devem inserir termos (ex: "Logothipo" em vez de "Logotipo", "Photoshopz" em vez de "Photoshop") e acertar a correspondência do Tag no resultado tolerante se não encontradas origens diretas, com feedback claro de UI exibida pelo componente padronizado de aviso.
