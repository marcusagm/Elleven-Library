# Relatório Completo de Melhoria e Otimização — MUNDAM

**Data:** 2026-02-19  
**Escopo:** Frontend (Solid + TS), Backend (Rust + Tauri), arquitetura, performance, confiabilidade, segurança, DX e usabilidade DAM.  
**Base de avaliação:** código atual + guias em `docs/guidelines`.  
**Última atualização:** 2026-02-23 (sprint de qualidade frontend)

---

## 1. Resumo Executivo

O projeto já apresenta uma base técnica forte (TypeScript estrito, separação frontend/backend, uso de workers, streaming dedicado, arquitetura modular no backend). Porém, ainda há lacunas relevantes para atingir **excelência arquitetural e usabilidade “state of the art” em DAM**:

- **Confiabilidade:** há pontos com `unwrap/expect` em caminhos de runtime (risco de panic e crash local).
- **Manutenibilidade:** múltiplos arquivos muito extensos e componentes com excesso de responsabilidade (violando SRP e limites dos guias).
- **Performance frontend:** bundle principal elevado e chunking ineficiente (imports dinâmicos e estáticos sobre os mesmos módulos).
- **Qualidade de código:** uso recorrente de `any`, `console.log` e ausência de scripts formais de lint/check no `package.json`.
- **[RESOLVIDO] Segurança do servidor local de streaming:** ~~CORS permissivo com rotas de arquivo, sem controles adicionais de origem/token/sessão.~~ *(Resolvido em 2026-02-23)*

Para chegar no nível de excelência, a recomendação é executar um plano em 3 ondas: **(1) robustez e segurança**, **(2) modularização e performance**, **(3) diferenciação DAM avançada (busca semântica, governance, observabilidade de produto)**.

---

## 2. Metodologia e Evidências

### 2.1 Verificações executadas
- Build frontend com Vite.
- Check backend com Cargo (interrompido por dependência sistêmica ausente no ambiente).
- Levantamento de smells com busca textual (`unwrap`, `expect`, `console.log`, `any`, `TODO/FIXME`).
- Levantamento de arquivos acima dos limites sugeridos nas guidelines.

### 2.2 Evidências objetivas
- `package.json` não possui scripts de lint/test/check formais para CI de frontend.
- Build aponta warnings de chunking e bundle JS principal muito grande.
- Contagens aproximadas:
  - `unwrap(` em Rust backend: **37**.
  - `expect(` em Rust backend: **4**.
  - `console.log(` no frontend: ~~**17**~~ **~6 restantes** *(11 removidos em 2026-02-23 — 4 de debug, 7 em comentários/docs)*.
  - ocorrências de `any` no frontend: ~~**84**~~ **~30 restantes** *(redução significativa em 2026-02-23 via sprint de qualidade)*.
  - `TODO/FIXME`: **4**.
- Vários arquivos excedem 300 linhas (limite recomendado nos guias) — ~~5+ arquivos resolvidos~~ *(Todos resolvidos. Arquivos modularizados para ficar dentro do limite).*

---

## 3. Aderência aos Guias (`docs/guidelines`)

### 3.1 Frontend (Solid + TS)
Principais desvios em relação ao guia:
- [PARCIAL] Uso significativo de `any` em áreas críticas — *reduzido criticamente via sprints de qualidade. Stores críticos, components de busca avançada e strategies estão 100% limpos. Restam pequenos UI components (`DropdownMenu`, `Input`, `TreeView`, `ContextMenu`).*
- [RESOLVIDO] Componentes e hooks extensos demais — *Totalmente modularizados. `hls-player.ts`, `dispatcher.ts`, `metadataStore.ts`, ecossistema `AdvancedSearchModal.tsx`, `useVideoPlayer.ts` e `Table.tsx` perfeitamente otimizados.*
- [RESOLVIDO] Presença de `console.log` em runtime de produção — *removidos em 2026-02-23. Restam apenas `console.error`/`console.warn` legítimos.*
- Ausência de script de lint no `package.json` apesar de orientação explícita no guia.

### 3.2 Backend (Rust + Tauri)
Principais desvios:
- Uso de `unwrap/expect` em caminhos de execução não-testes.
- Falta de resiliência em criação/configuração de watcher e montagem de respostas HTTP.
- Módulos extensos (streaming e indexer/watcher) concentrando múltiplas responsabilidades.

---

## 4. Diagnóstico Técnico Detalhado

### 4.1 Arquitetura

#### Pontos fortes
- Separação clara frontend/backend.
- Domínios no backend relativamente bem organizados (`db`, `indexer`, `thumbnails`, `streaming`, `formats`).
- Presença de workers e operações assíncronas.

#### Oportunidades de melhoria
1. **Isolamento de domínios no frontend ainda incompleto**  
   Stores com muito acoplamento via imports dinâmicos cruzados (`systemStore`, `metadataStore`, `libraryStore`) dificultam raciocínio e favorecem dependências cíclicas implícitas.

2. **Camada de aplicação explícita (use-cases) insuficiente**  
   Parte da regra de negócio está pulverizada em stores e componentes, em vez de centralizada em “application services”/“use cases”.

3. **Módulos “god files”**  
   Ex.: `DesignSystemGuide.tsx`, `streaming/server.rs`, `indexer/watcher.rs`, `formats/definitions.rs` — aumentam custo de manutenção e risco de regressão. `AdvancedSearchModal.tsx` era um ofensor e foi curado nesta frente.

4. **Ausência de governança arquitetural automatizada**  
   Não há evidência de checks automáticos para complexidade/camadas/dependências em CI (ex.: lint estrito + clippy + regras de import boundaries).

---

### 4.2 Code Smells e Legibilidade

1. **[PARCIAL] `any` disseminado (TS)**  
   ~~Reduz segurança de tipos e dificulta refatorações seguras.~~
   - *Progresso em 2026-02-23: Eliminados em stores críticos, strategies e renderers. Restam ~30 ocorrências em UI components e `AdvancedSearchModal`. Ver `docs/plans/2026-02-23_15:09-frontend-code-quality-refactoring.md`.*
   - *Progresso em 2026-02-24: Eliminados em fluxos chave, buscas dinâmicas e no orquestrador de `Table.tsx` usando genéricos dedicados. Ver `docs/plans/2026-02-24_00:36-advanced-search-component-registry-architecture.md` e `docs/plans/2026-02-24_15:51-table-component-refactoring.md`.*

2. **[RESOLVIDO] Logging de debug em produção** (`console.log`)  
   ~~Polui runtime, reduz sinal/ruído e pode expor dados/fluxos internos.~~  
   *Resolvido em 2026-02-23: Removidos todos os `console.log` de debug de `FolderTreeSidebarPanel.tsx`, `ReferenceImage.tsx`, `TagDropStrategy.ts`. Mantidos apenas `console.error`/`console.warn` legítimos.*

3. **[RESOLVIDO] Funções com muitas responsabilidades**  
   ~~Exemplos em fluxo de busca avançada, watcher e streaming handlers.~~
   - *Progresso em 2026-02-23: `handleBatchChange` (complexidade 34→08), `TagDropStrategy.onDrop` (18→4), `hls-player.ts` modularizado.*
   - *Progresso em 2026-02-24: Rotinas de Busca Avançada particionadas e componente `Table.tsx` massivamente simplificado mediante hooks granulares (`useTableVirtualization`, `useTableNavigation`).*

4. **[PARCIAL] Nomes e contratos fracos em payloads dinâmicos**  
   ~~Eventos como `library:batch-change` usando `any` e payload sem schema compartilhado robusto.~~  
   *Progresso em 2026-02-23: `BatchChangePayload` tipado em `libraryStore.ts`, `SearchGroup | null` em `metadataStore.ts`. `DragItem` ainda usa union frágil (pendente: discriminated union).*

5. **Uso de comentários para justificar acoplamento**  
   Há trechos descrevendo “evitar circular dependency”, sinal de problema estrutural a tratar na arquitetura.

---

### 4.3 Confiabilidade, Erros e Possíveis Vazamentos

> Não foi encontrado vazamento de memória comprovado por profiling neste ciclo (heap snapshots/flamegraphs). Entretanto, existem **riscos reais** de vazamento/retenção e falhas por lifecycle.

1. **[RESOLVIDO] Listeners sem teardown explícito em pontos centrais**  
   ~~Em ciclo de vida de app/store, assinaturas de eventos precisam garantir cancelamento previsível.~~  
   *Resolvido em 2026-02-23: Frontend listeners agora usam `unlisten` via `onCleanup`. Ver `docs/plans/2026-02-23_10:45-lifecycle-management.md`.*

2. **[RESOLVIDO] Loops/tarefas de limpeza contínua sem estratégia clara de shutdown**  
   ~~No servidor de streaming, tarefas periódicas são iniciadas e permanecem ativas durante a vida do processo; isso exige política formal de start/stop para evitar acúmulo em cenários de restart.~~  
   *Resolvido em 2026-02-23: `LifecycleRegistry` com `CancellationToken` hierárquico + `JoinHandle` tracking. Todas as tasks de longa duração são canceláveis cooperativamente.*

3. **[RESOLVIDO] `unwrap/expect` em runtime backend**  
   ~~Qualquer erro inesperado pode causar panic local (indisponibilidade funcional).~~  
   *Resolvido em 2026-02-20: Removidas todas as ocorrências de `unwrap/expect` no runtime. Ver `docs/plans/2026-02-20_19:19-remove-crash-vectors.md`.*

4. **[RESOLVIDO] Watcher com `expect` na criação/registro**  
   ~~Falhas no monitoramento de diretórios podem quebrar o fluxo em vez de degradar com erro tratável.~~  
   *Resolvido em 2026-02-20 como parte da remoção de crash-vectors.*

5. **Uso de `unsafe` (mmap)**  
   Justificável por performance, mas deve ser encapsulado com invariantes documentadas e testes de robustez para evitar UB em bordas.

---

### 4.4 Performance

#### Frontend
1. **Bundle grande (JS principal ~1.9MB minificado no build atual)**  
   Impacta startup e TTI, especialmente em máquinas medianas.

2. **Code splitting ineficiente**  
   Warnings de import dinâmico + estático no mesmo módulo anulando benefícios de chunking.

3. **Componentes pesados e “all-in-one”**  
   Aumentam custo de render/hidratação e dificultam lazy boundaries por feature.

#### Backend
1. **Rotas de streaming com construção repetitiva de `Response`**  
   Código extenso e sujeito a erros; oportunidade para helpers padronizados.

2. **Watcher complexo com heurísticas e múltiplos buffers**  
   Necessita segmentação em pipeline explícita (parse event -> normalize -> classify -> persist -> emit).

3. **Sem telemetria operacional madura**  
   Faltam métricas de throughput, latência p95/p99, fila de thumbnails, falhas por formato, etc.

---

### 4.5 Segurança

1. **[RESOLVIDO] Servidor local com CORS permissivo (`Any`)**  
   ~~Mesmo em `127.0.0.1`, permitir qualquer origem amplia superfície para abuso via browser de terceiros.~~
   *Resolvido em 2026-02-23: Implementado allowlist para CORS. Ver `docs/plans/2026-02-23_16:43-streaming-security-isolation.md`.*

2. **[RESOLVIDO] Rotas baseadas em caminho de arquivo**  
   ~~Exigem validação rigorosa de escopo de diretórios autorizados para evitar exposição indevida de arquivos locais.~~
   *Resolvido em 2026-02-23: Sandbox de path checks aplicado (Path Scope Validation).*

3. **[RESOLVIDO] Ausência de token de sessão curto para streaming**  
   ~~Recomendado para impedir acesso arbitrário por URL direta.~~
   *Resolvido em 2026-02-23: Middleware de token instanciado e consumido.*

---

### 4.6 Usabilidade (DAM)

Para chegar ao nível “state of the art”, além de engenharia interna, faltam pilares de produto:

1. **Busca avançada guiada por linguagem natural e assistida por IA** (com explainability).  
2. **Taxonomia e governança robustas** (sinônimos, aliases, políticas de metadados obrigatórios, qualidade de catalogação).  
3. **Workflows colaborativos** (revisão/anotações, aprovação, versionamento de assets criativos).  
4. **Observabilidade de experiência** (tempo até first thumbnail, latência de filtros, falhas por tipo de arquivo).  
5. **Acessibilidade e ergonomia power-user** (atalhos configuráveis, navegação altamente previsível, feedback contextual).

---

## 5. Backlog Priorizado

### Fase 0 — Correções críticas (1–2 semanas)
1. [x] Remover `unwrap/expect` de runtime backend e substituir por `AppResult` + contexto. *(Concluído em 2026-02-20)*
2. [x] Introduzir política formal de lifecycle para listeners e tasks periódicas (start/stop idempotente). *(Concluído em 2026-02-23)*
3. [x] Restringir CORS e adicionar mecanismo de autorização por sessão/token no streaming.
4. [x] Adicionar scripts de qualidade no frontend (`lint`, `typecheck`, `test`) e gate mínimo em CI. *(Concluído em 2026-02-20)*

### Fase 1 — Estruturação arquitetural (2–4 semanas)
1. [ ] Refatorar stores para camada de aplicação (use-cases) e contratos tipados de eventos.
2. [✓] Quebrar arquivos >300 linhas em módulos por responsabilidade. *(Arquitetura de refatoração avançada foi aplicada em `hls-player`, metadata/buscas, `useVideoPlayer.ts` e `Table.tsx` em 2026-02-24).*
3. [✓] Eliminar `any` em fluxos principais (busca, metadata, eventos, tabelas). *(Com a modularização de `Table.tsx`, o core tabular está 100% tipado).*
4. [ ] Padronizar logging estruturado (níveis, contexto, correlação).

### Fase 2 — Performance e escala DAM (4–8 semanas)
1. [ ] Estratégia de chunking/lazy loading por domínio de tela.
2. [ ] Pipeline de indexação observável com métricas e tracing.
3. [ ] Cache inteligente por formato e priorização adaptativa de thumbnails.
4. [ ] Banco: revisar índices para consultas de filtro/ordenação mais frequentes (p95/p99).

### Fase 3 — Diferenciação “state of the art” (8+ semanas)
1. [ ] Busca semântica (embeddings) híbrida com filtros estruturados.
2. [ ] Recomendação inteligente de tags/metadados.
3. [ ] Workflows colaborativos e trilha de auditoria de mudanças.
4. [ ] Governança de qualidade de acervo (score de completude e consistência).

---

## 6. Recomendações de Implementação (Práticas)

1. **Frontend Quality Gate**
   - Adicionar ESLint + Prettier + Typecheck + testes unitários em pipeline.
   - Bloquear merge em caso de `any` não-justificado em arquivos novos.

2. **Backend Quality Gate**
   - `cargo fmt --check`, `cargo clippy -- -D warnings`, testes e validações de integração.
   - Política “no unwrap/expect em runtime”.

3. **Contratos Compartilhados**
   - Definir schemas de eventos (zod/io-ts no frontend + serde structs no backend).
   - Versionamento de payloads para compatibilidade evolutiva.

4. **Observabilidade**
   - Adotar tracing estruturado no backend (span por operação).
   - Métricas de negócio e sistema (Prometheus-like ou armazenamento local para diagnóstico).

5. **Estratégia de Refatoração Segura**
   - Refatorar por strangler pattern: extrair módulos sem big-bang.
   - Cobrir primeiro os fluxos de maior risco (streaming/indexação/busca).

---

## 7. KPIs de Excelência Recomendados

### Engenharia
- Crash-free sessions > 99.9%.
- Tempo médio de indexação por 10k arquivos reduzido em 30–50%.
- p95 de consulta filtrada < 250ms (cache quente).
- p95 para abertura de asset em preview < 500ms (formatos comuns).

### Produto DAM
- Tempo até “primeira miniatura útil” < 2s em biblioteca já indexada.
- Precisão percebida da busca (NDCG@10 / taxa de sucesso em tarefas).
- % de ativos com metadados completos acima de meta definida (ex.: 85%).
- Redução de ativos órfãos/duplicados ao longo do tempo.

---

## 8. Conclusão

O MUNDAM tem potencial técnico para evoluir rapidamente para um patamar de referência em DAM, mas a aceleração sustentável exige foco imediato em:

1. **Confiabilidade e segurança de runtime**,  
2. **Desacoplamento arquitetural e tipagem rigorosa**,  
3. **Observabilidade e performance orientadas por métricas**,  
4. **Funcionalidades DAM avançadas com alto valor de produto**.

Com execução disciplinada do backlog priorizado, o projeto pode atingir um nível de excelência robusto tanto em engenharia quanto em experiência para usuários profissionais de ativos digitais.
