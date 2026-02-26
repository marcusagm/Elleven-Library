# Sprint 0: Fundamentos e Guardrails (Infraestrutura)

**Data:** 2026-02-26  
**Status:** Concluído ✅  
**Objetivo:** Estabelecer a base técnica, contratos e ferramentas de segurança arquitetural para suportar a migração das stores e ações sem riscos de regressão.

---

## 🏗️ 1. Infraestrutura e Contratos

- [x] **Definição de Tipos Globais:**
    - Criar `src/core/types/actions.ts`.
    - Definir `ActionResult<DataType, ErrorType>` para padronizar o retorno de todas as ações.
    - Definir `BaseError` com códigos padronizados (`VALIDATION_ERROR`, `IO_ERROR`, etc.).
- [x] **Integração Zod:**
    - Instalar `zod` e `zod-validation-error`.
    - Criar política de uso de schemas (onde residem, como são nomeados) em `docs/guidelines/core-architecture.md`.
- [x] **Factory de Actions:**
    - Implementar utilitário `createSecureAction` em `src/core/utils/actions.ts` para automatizar:
        - Validação de Input Schema.
        - Tratamento de erro padronizado.

## 🛡️ 2. Guardrails e Governança

- [x] **Configuração de Lint Arquitetural:**
    - Adicionar regras de ESLint (`import/no-restricted-paths`) para proibir:
        - Import de `tauriService` em diretórios `src/components/`.
        - Uso de `toast` dentro de `src/core/store/`.
        - Ciclos explícitos entre stores (`import/no-cycle`).
- [x] **Baseline de Métricas:**
    - Criar script `.agent/scripts/count_any.py` para contar `any` remanescentes em `src/core`.
    - Baseline inicial em `src/core`: **7** `any` identificados.
- [x] **Estratégia de Rollout:**
    - Implementar utilitário de Feature Flags simples (`src/core/system/featureFlags.ts`) para habilitar gradualmente os novos fluxos.

## 📋 3. Critérios de Aceite (DoD)

1. [x] `ActionResult` e `BaseError` implementados e documentados (TSDoc).
2. [x] `zod` configurado e primeiro schema de teste (`test-schema.ts`) funcionando com Vitest.
3. [x] Script de contagem de `any` operacional.
4. [x] Regras de lint arquitetural validadas via `npm run lint`.
5. [x] Estrutura de `src/core/types` e `src/core/utils` criada.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Complexidade Excessiva no `createSecureAction`** | Manter a primeira versão simples, apenas com validação de input e retorno padronizado. |
| **Resistência ao Lint Estrito** | Aplicar o lint apenas aos arquivos novos/tocados inicialmente (gradual). |
| **Overhead de Performance (Zod)** | Validar apenas payloads complexos e disparados por input de usuário. |
