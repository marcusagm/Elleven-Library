# Sprint 4: Advanced Metadata Batch Operations & Fuzzy Search

**Data:** 2026-03-02
**Status:** Planejado
**Data e hora da conclusão:** -

## 📌 Objetivo
Avançar com recursos definitivos para manipulação em massa e buscas de tolerância alta ("Search & Tagging") preenchendo os gaps de Usabilidade Profissional da Fase 2. Isso assegura que governar o acervo com comandos complexos para múltiplos objetos ou descrições imprecisas seja fluido para o *Power-User*.

## 🛠 Tarefas de Implementação

### 1. Masterização no Batch Tagging (`MultiInspector.tsx`)
- **Escopo:** Interações dinâmicas sobre metadados (edições, deleções, transições complexas de *classes*) operando confiavelmente sob milhares de fotos selecionadas de uma só vez, sem lock in na thread.
- **Ações (Frontend & Backend):**
  - **Frontend:** Desenhar `MultiInspector.tsx` através de Atomic Composition ou Compound Components para exibir estados "Indeterminados" (elementos onde nem todos da seleção possuem determinada tag). As actions subjacentes jamais operarão em map-loop sobre item único; chamarão abstrações unificadas no hook.
  - **Backend:** Desenvolver o comando `metadata_update_batch` com tratamento atômico SQLx (`pool.begin()`) que suportará deltas lógicos (ApplyTag, RemoveTag) otimizados em transação densa utilizando INSERT e DELETE vinculados.
  - O processamento longo informará progresso de etapas via pacote na Sprint 1 (Logging e Trace UI vinculados).
- **Validação:** Seleções imensas e arrastáveis (>10.000 imagens/vetoriais) podem ser tageadas sob a margem temporal p95 sem interface congelada e sem explosão de consumo de memória RAM. Ausência terminante de manipulação de string manual no TS e obediência total para tipagem (schemas *Zod* de payload).

### 2. Busca Tolerante Sintática & "Fuzzy Search"
- **Escopo:** Substituir as falhas de pesquisa cruas que dependam exclusivamente de sintaxe ortográfica perfeita, entregando algoritmos leves e inteligentes no catálogo (SQLite local).
- **Ações:**
  - Instanciar a extensão dedicada de distanciamento no SQLite (como `spellfix1`, `FTS5`, algorítmos atados de `Levenshtein` ou `Trigram` nativos). Manter implantação leve minimizando complexidade adicional pesada no OS host.
  - A lógica unifica-se à base de `QueryBuilder` existente (`src/db/search.rs`), em que na ativação nativa do frontend o usuário passa as diretrizes como "Buscar similaridades" com matching de precisão ou erro adaptativo.
  - O frontend refletirá os matches com score de semelhança (fuzzy logic) para garantir relevância no output da VirtualList e AdvancedSearchModal, descartando "ruídos".
- **Validação:** Usuários devem inserir termos (ex: "Logothipo" em vez de "Logotipo", "Photoshopz" em vez de "Photoshop") e acertar a correspondência do Tag no resultado tolerante se não encontradas origens diretas, com feedback claro de UI exibida pelo componente padronizado de aviso.
