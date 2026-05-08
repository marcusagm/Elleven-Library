# Wave 10 — Migração Final V1 → V2: Superioridade e Completude

**Data de criação:** 2026-03-17
**Status geral:** Pendente
**Sprint base referenciada:** Sprint 9.1 (principais regressões já corrigidas)

## Visão Geral

Esta wave consolida as últimas tarefas para que a V2 seja completa e **superior** à V1 em todos os aspectos. As sprints foram criadas com base em análise profunda do código-fonte atual de ambas as arquiteturas.

## Estado da Migração ao Criar Esta Wave

### ✅ Concluído nas waves anteriores (1-9)
- Estrutura hexagonal completa
- AssetLedger, EventBus, FormatRegistry
- ThumbnailWorker V2 (LIFO + FIFO + JoinSet paralelo)
- ColorWorker V2 (event-driven via ThumbnailGenerated)
- WatcherService (sensor.rs + debouncer.rs)
- HlsManager V2 (DashMap + cleanup + CancellationToken)
- 23 FormatProviders (vs 15 da V1)
- Todos os IPC de Indexer, Tags, SmartFolders, Assets, Search corrigidos
- Autenticação de streaming via StreamingSessionToken ✅

### ⚠️ Issues Identificadas e Escopo desta Wave

| Sprint                   | Prioridade | Tema                                                             |
| ------------------------ | ---------- | ---------------------------------------------------------------- |
| [10.1](sprint-10-1.md)   | 🔴 Alta     | Segurança CORS + Graceful Shutdown do Streaming Server           |
| [10.2](sprint-10-2.md)   | 🔴 Alta     | Indexer Paralelo (fan-out producer-consumer)                     |
| [10.3](sprint-10-3.md)   | 🟡 Média    | SAI v1 — MetadataCapability + PreviewCapability                  |
| [10.4](sprint-10-4.md)   | 🟡 Média    | SAI2 — Auditoria e paridade com V1                               |
| [10.5](sprint-10-5.md)   | 🟡 Média    | CorelDRAW (.cdr) — Auditoria e paridade                          |
| [10.6](sprint-10-6.md)   | 🟡 Média    | GIMP XCF — Auditoria + extract_dimensions                        |
| [10.7](sprint-10-7.md)   | 🟡 Média    | MDP (MediBang/FireAlpaca) — Dimensões + registro                 |
| [10.8](sprint-10-8.md)   | 🟡 Média    | Rebelle + Penpot — Completar extractors                          |
| [10.9](sprint-10-9.md)   | 🔴 Alta     | Settings IPC — get_cache_stats e comandos faltantes              |
| [10.10](sprint-10-10.md) | 🔴 Alta     | Áudio/Vídeo — Extensões legadas (dts, ac3, f4v, etc.)            |
| [10.11](sprint-10-11.md) | 🔴 Alta     | Color Worker — Corrigir erros de WebP inválido                   |
| [10.12](sprint-10-12.md) | 🟢 Baixa    | Features exclusivas V2 (color search, health check, maintenance) |

## Ordem de Execução Recomendada

```
1. Sprint 10.11 → Corrigir Color Worker (afeta todos os formatos) ✅
2. Sprint 10.9  → Corrigir Settings IPC (bloqueia UI) ✅
3. Sprint 10.10 → Paridade extensões áudio/vídeo (afeta inspector) ✅
4. Sprint 10.1  → Segurança CORS (critical para produção) ✅
5. Sprint 10.3 + 10.7 → SAI v1 + MDP (mais usados pelos usuários) ✅
6. Sprint 10.2  → Indexer paralelo (performance em bibliotecas grandes) ✅
7. Formatos restantes:
 - Sprint 10.4 (SAI 2) ✅
 - Sprint 10.5 (CorelDRAW) ✅
 - Sprint 10.6 (GIMP XCF)
 - Sprint 10.8 (Rebelle + Penpot) 
8. Sprint 10.12 → Features exclusivas V2 (polimento final)
```

## Como Medir Sucesso

- [ ] Zero erros no console ao usar o app com uma biblioteca real de 10k+ assets
- [ ] Todos os 23 FormatProviders geram thumbnails válidos (zero `Invalid Chunk header`)
- [ ] Scan de 10k arquivos completa em < 30s (vs minutos com scan serial)
- [ ] CORS bloqueia requests de origens não autorizadas
- [ ] Settings página funciona completamente sem erros
- [ ] V2 passa em todos os testes que a V1 passou (regressão zero)
