# Sprint 2.2: O Adaptador do Ledger (Infra SQLx)

**Status:** Concluído ✅
**Data e hora de inicio:** 2026-03-05T21:00:00Z
**Data da conclusão:** 2026-03-05T23:55:00Z

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Sujar as mãos implementando o lado de Inserts do CQRS. Conectar a Interface criada na sprint anterior ao `SQLx`, efetuando transações ACID no SQLite real para as tabelas vitais de Media/Assets.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Transações Reais:** [Concluído] O `SqliteAssetLedger` grava na tabela `v2_assets` real sem falhas de lock.
2. **Rolback Seguro:** [Concluído] Verificado via Rollback automático do dropped transaction em falhas de constraint.
3. **Persistência Completa:** [Concluído] `BatchCreate` e `UpdateAsset` cravando dados com carimbos auditáveis.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Criação do Adaptador
- [x] Criar o arquivo `src-tauri/src/infra/database/ledger.rs`.
- [x] Instanciar a struct `SqliteAssetLedger` com injeção de `SqlitePool` e `AppEventBus`.
- [x] Implementar a trait `TransactionalAssetLedger` mapeando `LedgerCommand` para SQL.

### 2. Controle Transacional Rigoroso
- [x] Implementado em `execute()` usando `self.pool.begin().await?` e garantindo commit atômico.

### 3. Emissão de Eventos Genuínos
- [x] Eventos de domínio (AssetCreated, MetadataUpdated, FsPathDeleted) publicados apenas após o sucesso do commit.

### 4. Registro no Container
- [x] Registrado via `app.manage` em `lib.rs` como `Arc<dyn TransactionalAssetLedger>`.

---

## 🚀 Resumo das Atividades e Resultados

### O que foi realizado
- **Draft e Implementação do SqliteAssetLedger**: Implementação completa do adaptador real substituindo o Mock da Sprint 2.1.
- **Suporte a Batch indexing**: Implementação do comando `BatchCreate` para lidar com a carga inicial de centenas de arquivos sem gargalos de I/O.
- **Integração total do Watcher**: O Watcher agora utiliza o Ledger para todas as persistências em tempo real (adições, renomeações e deleções).
- **Trilha de Auditoria (CQRS)**: Cada operação é registrada na tabela `v2_asset_operations_log`, garantindo rastreabilidade total das mudanças no sistema.

### Dificuldades e Soluções
- **IDs V1 vs V2**: O Watcher original trabalhava com IDs `i64` do banco legado, enquanto o Ledger V2 utiliza `String` (UUID).
    - *Solução*: Refatoração do `LedgerCommand` para aceitar `PathBuf` como lookup opcional, permitindo que o Watcher interaja com o Ledger via caminho do sistema de arquivos quando o ID V2 ainda não é conhecido.
- **Lifetimes e Async**: Problemas com o borrow checker ao capturar o Ledger `Arc` dentro de tasks `tokio::spawn` no Watcher.
    - *Solução*: Clonagem explícita do `Arc` fora do escopo da task para garantir validade `'static`.
- **Nullable types no SQLx**: A macro `query!` inferia IDs como opcionais, causando erros de tipo no Rust.
    - *Solução*: Uso de hints explicitos do SQLx (`id as "id!"`) para garantir tipos não-nulos.

### Entrega Adicional (Além do Escopo)
- **Flexibilidade de Lookup**: O Ledger foi desenhado para ser resiliente, resolvendo IDs de assets automaticamente a partir de caminhos de arquivos, o que facilitou drasticamente a integração com o Watcher legado.
- **Eventos de Domínio Refinados**: Adição do evento `AssetMetadataUpdated` para notificar mudanças de nome/caminho de forma precisa.

---

## 💡 Notas para o Desenvolvedor / Agente
> O `SqliteAssetLedger` agora é a única fonte de verdade para mutações de assets. A infraestrutura V2 está sólida e pronta para a próxima fase de processamento intensivo (extração de thumbnails e metadados via workers).
