# Sprint 0: Fundamentos e Guardrails (Infraestrutura)

**Data:** 2026-02-26  
**Status:** Planejado  
**Objetivo:** Estabelecer a base técnica, contratos e ferramentas de segurança arquitetural para suportar a migração das stores e ações sem riscos de regressão.

---

## 🏗️ 1. Infraestrutura e Contratos

- [ ] **Definição de Tipos Globais:**
    - Criar `src/core/types/actions.ts`.
    - Definir `ActionResult<TData, TError>` para padronizar o retorno de todas as ações.
    - Definir `BaseError` com códigos padronizados (`VALIDATION_ERROR`, `IO_ERROR`, etc.).
- [ ] **Integração Zod:**
    - Instalar `zod` e `zod-validation-error`.
    - Criar política de uso de schemas (onde residem, como são nomeados).
- [ ] **Factory de Actions:**
    - Implementar utilitário `createSecureAction` (opcional, mas recomendado) para automatizar:
        - Validação de Input Schema.
        - Tratamento de erro padronizado.
        - Logs de auditoria em modo DEV.

## 🛡️ 2. Guardrails e Governança

- [ ] **Configuração de Lint Arquitetural:**
    - Adicionar regras de ESLint (ou custom script) para proibir:
        - Import de `tauriService` em diretórios `src/components/`.
        - Uso de `toast` dentro de `src/core/store/`.
        - Ciclos explícitos entre stores.
- [ ] **Baseline de Métricas:**
    - Criar script/comando para contar `any` remanescentes em `src/core`.
    - Estabelecer baseline de performance para o Viewport (latência de render).
- [ ] **Estratégia de Rollout:**
    - Implementar utilitário de Feature Flags simples (`featureFlags.ts`) para habilitar gradualmente os novos fluxos (`new-actions-search`, etc.).

## 📋 3. Critérios de Aceite (DoD)

1. [ ] `ActionResult` e `BaseError` implementados e documentados.
2. [ ] `zod` configurado e primeiro schema de teste funcionando.
3. [ ] Script de contagem de `any` operacional.
4. [ ] Regras de lint arquitetural validadas.
5. [ ] Estrutura de `src/core/types` criada.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Complexidade Excessiva no `createSecureAction`** | Manter a primeira versão simples, apenas com validação de input e retorno padronizado. |
| **Resistência ao Lint Estrito** | Aplicar o lint apenas aos arquivos novos/tocados inicialmente (gradual). |
| **Overhead de Performance (Zod)** | Validar apenas payloads complexos e disparados por input de usuário. |
