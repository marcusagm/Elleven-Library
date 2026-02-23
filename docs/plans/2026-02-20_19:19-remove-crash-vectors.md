# Remove Crash-Vectors (unwrap/expect) from Backend

## Objetivo
Remover completamente o uso de `unwrap()` e `expect()` no código Rust do backend para prevenir panics e crashes na aplicação (ex: arquivos corrompidos, ausência de banco de dados, falhas de I/O silenciosas). Vamos aderir a uma abordagem mais segura e idiomática.

## Metodologia Híbrida

1. **Camada Base (Traits de Extensão):**
   - Criar `ResultExt` e `OptionExt` (ou um trait unificado `ErrorContext`) no módulo `error.rs`.
   - Implementar a função `.context("Mensagem")` para mapeamento facilitado (imitando The `anyhow::Context` format).
2. **Barreira Linter (Clippy):**
   - Adicionar as regras de `#![deny(clippy::unwrap_used)]` e `#![deny(clippy::expect_used)]` ao nível da crate no `lib.rs` ou `main.rs`.
3. **Migração Iterativa:**
   - Para não quebrar o build todo instantaneamente, adicionar `#[allow(clippy::unwrap_used, clippy::expect_used)]` no topo dos módulos que ainda não foram refatorados, ou refatorar módulo a módulo diretamente.
4. **Substituição:**
   - Trocar `.unwrap()` e `.expect("msg")` por `.context("msg")?` em métodos que já retornam `AppResult`.
   - Modificar a assinatura das funções que retornavam valores sem encapsulamento para passar a retornar `AppResult<T>`.

## Passos da Implementação (Roadmap)

- [x] Fase 1: Extensões do Error Framework `src-tauri/src/error.rs`.
- [x] Fase 2: Ativar linters parciais/totais no diretório de sources.
- [x] Fase 3: Migração do subsistema `formats/` e sub-extratores.
- [x] Fase 4: Migração do subsistema `thumbnails/`.
- [x] Fase 5: Migração do subsistema `library/` e `indexer.rs`.
- [x] Fase 6: Migração do subsistema `media/` e `streaming/`.
- [x] Fase 7: Migração do subsistema `db/`.
- [x] Fase 8: Revisão final e remoção total dos `#[allow(...)]`.
