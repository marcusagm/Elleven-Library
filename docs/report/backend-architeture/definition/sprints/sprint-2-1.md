# Sprint 2.1: Traits do Domínio e Modelos (Commands)

**Status:** Concluído
**Data e hora de inicio:** 2026-03-05 19:20
**Data da conclusão:** 2026-03-05 19:40

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Estabelecer a Fronteira de Mutação (O "C" do CQRS). Criar as Data Transfer Objects (DTOs) que representam as intenções de mudança de estado e a Interface Abstrata (Trait) do AssetLedger que as recebe. Esta sprint não toca no banco de dados.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Modelagem Pura:** Os Commands (`CreateAssetCommand`, `UpdateMetadataCommand`) devem ser instanciáveis como structs imutáveis.
2. **Trait Definida:** A interface `TransactionalAssetLedger` deve estar compilando, assinando as intenções de mutação e devolvendo `AppResult<Asset>`.
3. **Mock Ledger:** Uma implementação temporária `MockAssetLedger` em memória (ex: usando um `HashMap`) deve processar um `CreateAssetCommand` e simular um evento emitido no `EventBus` construído na Fase 1.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição dos Commands
- [x] Criar `src-tauri/src/core/ledger/command.rs`.
- [x] Implementar a struct `CreateAssetPayload` (através do enum `LedgerCommand`).
- [x] Implementar a struct `UpdateTagsPayload` (através do enum `LedgerCommand`).

### 2. Contrato do Ledger (Porta Hexagonal)
- [x] Criar `src-tauri/src/core/ledger/port.rs`.
- [x] Definir `#[async_trait] pub trait TransactionalAssetLedger: Send + Sync`.
- [x] Adicionar os métodos centrais: `async fn execute(&self, cmd: LedgerCommand) -> AppResult<Asset>`.

### 3. Integração com o EventBus
- [x] O `MockAssetLedger` criado para testes deve injetar uma referência do `TokioEventBus` (via `AppEventBus`).
- [x] No fim do método `register_asset` (no `execute`), o Mock deve invocar `self.bus.publish(DomainEvent::AssetCreated)`.

### 4. Bateria de Testes Unidimensionais
- [x] Escrever Teste Unitário (`#[tokio::test]`) que inicializa o Bus, o MockLedger e submete um Command.
- [x] Fazer uma Asserção (Assert) de que o Bus recebeu o evento, validando a dança entre Domínio e Mensageria.

---

## 💡 Notas para o Desenvolvedor / Agente
> O Ledger é o guardião da consistência. Nenhum Command Handler externo deve criar eventos aleatoriamente; o Ledger é quem aplica a mudança atômica e avisa o restante do sistema. Na próxima sprint, trocaremos o "Mock InMemory" por SQLite.
