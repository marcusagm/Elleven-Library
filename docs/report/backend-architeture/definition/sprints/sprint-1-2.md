# Sprint 1.2: O Motor do EventBus Assíncrono

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 1:** Fundação & Observabilidade (Core Mínimo)
**Objetivo:** Introduzir e testar o Canal Desacoplado de Comunicação do sistema (`tokio::sync::broadcast`) que rege a Arquitetura Orientada a Eventos (EDA).

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Publicador e Assinante (Pub/Sub):** Mudar estado sem chamar função direta. O módulo A deve conseguir disparar um `DomainEvent::AssetDiscovered`, e o módulo B recebê-lo em loop autônomo.
2. **Backpressure:** Demonstrar que o sistema suporta rajadas (bursts) emuladas contínuas do `tokio::mpsc/broadcast` sem travar a main thread do Tokio, usando testes (Rust Tests) ou CLI logs.
3. **Injeção de Dependências:** O Tauri State manager (`app_handle.manage()`) deve injetar corretamente a interface `EventBus` nos comandos sem *Borrow Checker* panics.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição do EventBus (Core)
- [ ] Criar o arquivo de Definição do EventBus em `src-tauri/src/core/events/mod.rs` ou `core/events/bus.rs`.
- [ ] Construir a trait Padrão: `pub trait EventBus: Send + Sync { fn publish... fn subscribe... }`.
- [ ] Montar a Enumeração canônica `DomainEvent`, blindada sob `#[derive(Clone, Debug, Serialize)]`, como por exemplo as chaves: `AssetDiscovered`, `ThumbGenerated`, `ScanStarted`.

### 2. Implementação do Adaptador (Infra)
- [ ] Em `src-tauri/src/infra/events/tokio_bus.rs`, instanciar o `tokio::sync::broadcast::channel(2048)`.
- [ ] Lidar adequadamente com o erro passivo `SendError` e rastrear no log (usando o `tracing::debug!`) caso não haja ouvintes vivos no momento do Publish.
- [ ] Garantir que múltiplos recebedores (Receiver Clones) ouçam simultaneamente a mesma emissão.

### 3. Loop de Escuta Paralelo (Feature)
- [ ] Em `main.rs` na inicialização do Tauri, instanciar a infra `TokioEventBus`.
- [ ] Injetá-la no Tauri: `app.manage(Arc::new(bus) as Arc<dyn EventBus>)`.
- [ ] Fazer um `tokio::spawn(async move { loop { ... } })` bobo de testes para simular um "Worker Fantasma", apenas dando print/tracing que consumiu a mensagem "AssetDiscovered".

### 4. Cobertura Sólida
- [ ] Escrever Teste Unitário (`#[tokio::test]`) garantindo isolamento da infraestrutura: "Se eu publico 10 mil eventos, o Subscriber no fim da pool leu os 10 mil perfeitamente".

---

## 💡 Notas para o Desenvolvedor / Agente
> Preste atenção redobrada aos escopos variáveis. Quando fechar o emissor e os assinantes, use Arcs e Clones inteligentes. O pilar do Mundam está em nunca os módulos de domínio chamarem uns aos outros diretamente de agora em diante. Tudo flui pelo Bus.
