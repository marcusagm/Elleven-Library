# Análise Crítica do Plano de Refatoração de Actions/Stores

**Data:** 2026-02-26  
**Documento analisado:** `docs/report/2026-02-26_plano_refatoracao_actions_stores.md`  
**Objetivo desta análise:** avaliar coerência, apontar lacunas e riscos, propor melhorias e revisar a ordem de execução por sprint.

---

## 1) Verificação de coerência do plano

O plano é **coerente no objetivo macro** (UI presentacional + mutação apenas por actions + validação de payload com schema) e está alinhado com práticas de arquitetura orientada a domínio em front-end. Há consistência entre:

- objetivo declarado (desacoplamento UI/store),
- problemas mapeados em componentes concretos,
- e ações propostas para cada interação.

### Pontos fortes de coerência

1. **Direção arquitetural clara e uniforme**: todas as interações convergem para o mesmo padrão de fluxo unidirecional com validação de payload.
2. **Rastreabilidade por domínio**: cada interação liga componentes, hooks e stores específicos, reduzindo ambiguidade sobre “onde mexer”.
3. **Preocupação com tipagem e runtime safety**: uso de Zod, eliminação de `any`, atenção a dependências circulares e contratos entre camadas.
4. **Cobertura ampla do produto**: viewport, busca, inspector, settings, DnD e eventos globais foram considerados.

### Incoerências internas

1. **Sobreposição entre Interações e Fases**: as fases (1–5) não “amarram” claramente as 10 interações. Isso pode gerar execução fora de ordem lógica (ex.: começar Interação 3 sem baseline pronta da Fase 1).
2. **Conflito de ownership de responsabilidades**: em alguns trechos, ora se propõe colocar lógica na store, ora em Domain Service, ora em hook. Falta critério explícito de corte (o que fica em cada camada).
3. **Padrão de eventos ainda indefinido**: o plano cita `Domain Event Dispatcher`, mas não define contrato mínimo de evento, garantias (idempotência, ordering), nem política de consumo.
4. **DoD global forte, porém não incremental**: critérios de aceite aparecem no fim, sem “mini-DoD” por interação/sprint.

---

## 2) Pontos falhos (riscos práticos)

### 2.1 Falhas de execução

1. **Escopo muito grande por interação**: várias interações envolvem UI + store + serviços + persistência + notificações ao mesmo tempo.
2. **Ausência de estratégia de migração gradual**: falta definir modo “compat” (legacy + novo fluxo coexistindo com feature flags).
3. **Dependência cruzada silenciosa**: interações 3, 4, 8 e 10 compartilham `metadataStore`/tags/busca e podem colidir se executadas em paralelo.
4. **Sem baseline de observabilidade**: não há plano para medir regressão de performance, taxa de erro de actions, ou falhas de validação de schema.

### 2.2 Falhas técnicas

1. **Validação somente de payload é insuficiente**: para worker/DnD/eventos também é necessário validar **resultado/response** e envelopes de erro.
2. **Sem política de versionamento de schemas**: alterações futuras de payload podem quebrar chamadas existentes.
3. **Sem estratégia de concorrência**: ações assíncronas críticas (tags, batch change, smart folders) podem sofrer race conditions.
4. **Sem padronização de erro por domínio**: faltam códigos e mapeamento de erro para UI (ex.: `VALIDATION_ERROR`, `IO_ERROR`, `CONFLICT_ERROR`).

### 2.3 Falhas de governança

1. **Não define owners por trilha** (core, search, tags, viewport, settings).
2. **Não define cadence de revisão arquitetural** (RFC curta por interação crítica).
3. **Não define política de rollback** por sprint.

---

## 3) Melhorias que o plano não cobre (e deveria cobrir)

## 3.1 Arquitetura e contratos

- **ADR/RFC por decisão estrutural**: registrar decisões como `store vs service vs hook`, padrão de evento e contrato de action.
- **Contrato único de action** (`ActionResult<TData, TError>`): padronizar sucesso/falha e eliminar retorno ad-hoc.
- **Schema de entrada e saída**: `inputSchema` + `outputSchema` + `errorSchema` em actions críticas.
- **Versionamento de payload/eventos**: campo `version` para fluxos sensíveis (worker, DnD, persistência).

## 3.2 Qualidade e segurança

- **Pirâmide de testes por interação**:
  - unit para services/selectors,
  - contract tests para actions + schemas,
  - smoke e2e para fluxos críticos.
- **Métricas obrigatórias por sprint**:
  - contagem de `any` remanescentes,
  - cobertura de actions com schema,
  - tempo de render/scroll no viewport,
  - taxa de erro por tipo de action.
- **Lint arquitetural automatizado**:
  - proibir import de `tauriService` em UI,
  - proibir `toast` em store,
  - detectar ciclos entre stores.

## 3.3 Estratégia de rollout

- **Feature flags por domínio** (`new-actions-search`, `new-actions-tags`, etc.).
- **Plano de migração “strangler”**: adaptar handlers antigos para chamar as novas actions antes da remoção total do legado.
- **Plano de rollback**: script/checklist para retorno rápido por interação.

## 3.4 Performance e concorrência

- **Política de cancelamento e deduplicação** para ações assíncronas (AbortController, request key).
- **Controle transacional** para mutações encadeadas (especialmente tags, pastas e smart folders).
- **Budget de performance do viewport** com metas explícitas (latência de scroll, tempo de layout worker).

---

## 4) Análise crítica da ordem de interação/sprints

A ordem atual começa por Viewport/Seleção e deixa DnD no fim. Em termos de dependência real, isso é arriscado, porque DnD impacta diretamente seleção, tags, library e viewport. Além disso, busca/tags/inspector compartilham bastante domínio de metadados.

### Problemas na ordem atual

1. **Interação 1 depende de decisões da Interação 10** (DnD).
2. **Interações 3, 4 e 8 competem pela mesma base (`metadataStore`) cedo demais**.
3. **Interação 9 (viewport engine) pode causar regressões amplas se feita sem guardrails e métricas prévias**.

### Ordem sugerida (mais segura)

#### Sprint 0 — Fundamentos e guardrails (pré-requisito)
- Definir `ActionResult`, erro padrão e policy de schemas.
- Configurar lint arquitetural e métricas mínimas.
- Criar feature flags e estratégia de rollback.

#### Sprint 1 — Eventos globais + Settings (baixo risco, alto ganho)
- Interação 7 (Status/Eventos globais).
- Interação 6 (Settings/Preferências).

> Justificativa: remove acoplamentos óbvios (`window.dispatchEvent`, I/O em componente) e estabelece padrão de action com risco controlado.

#### Sprint 2 — Library base + Selection
- Interação 2 (Navegação/Biblioteca).
- Interação 1 (Seleção/Viewport superficial).

> Justificativa: consolida mutações centrais de dados e prepara terreno para busca/tags.

#### Sprint 3 — Tags + DnD (em conjunto)
- Interação 8 (Tags/hierarquia).
- Interação 10 (DnD).

> Justificativa: DnD e tags são acoplados; separar cria retrabalho de contratos.

#### Sprint 4 — Busca avançada + Inspector
- Interação 3 (Busca/Smart folders).
- Interação 4 (Inspector).

> Justificativa: ambos dependem de metadados estáveis e contratos já maturados.

#### Sprint 5 — ItemView + Viewport engine
- Interação 5 (ItemView).
- Interação 9 (Viewport/Workers).

> Justificativa: mudanças com maior risco de performance e UX devem ocorrer após estabilização dos contratos e observabilidade.

---

## 5) Critérios de qualidade por sprint (recomendado)

Para reduzir risco, cada sprint deve encerrar com um mini-DoD:

1. **Sem regressão crítica em smoke tests do domínio alterado**.
2. **100% das novas actions com schema de input (e output quando aplicável)**.
3. **Nenhuma chamada a I/O em componentes tocados no sprint**.
4. **Sem novos `any` em arquivos alterados**.
5. **Métricas mínimas coletadas e comparadas com baseline**.

---

## 6) Conclusão executiva

O plano está bem direcionado e tecnicamente maduro na intenção, mas ainda **pouco operacional** para execução sem risco. O maior ganho agora não está em aumentar o escopo técnico, e sim em:

1. Fortalecer contratos (actions/eventos/erros),
2. Definir migração incremental com feature flags,
3. Reordenar interações por dependência real,
4. Medir regressão continuamente.

Com essas correções, o plano passa de “bom diagnóstico arquitetural” para “roteiro executável de transformação em produção”.
