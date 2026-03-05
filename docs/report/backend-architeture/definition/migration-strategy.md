# Estratégia de Migração (Foco no Desenvolvimento Ágil)

## 1. Contexto Atual: Desenvolvimento Solitário (Zero Usuários)

A abordagem tradicional de migrações complexas (como o *Strangler Fig Pattern* estrito) é ideal para produtos faturando em produção e com bancos de dados de clientes reais, onde o risco de perda de dados deve ser diluído ao longo de meses.

No entanto, como o Mundam está em fase de desenvolvimento pré-lançamento (V0) e liderado por 1 único desenvolvedor, adotar um modelo tão precavido criaria uma burocracia paralisante (*Overhead*). Assim, a nossa **Estratégia de Migração será Agressiva, Direta, mas Contratual**.

O Foco: **Substituição rápida do motor interno em Rust, usando o Frontend (Solid.js) atual como suíte de testes E2E involuntária.**

---

## 2. A Lei de Ouro: "A API do Tauri é a Fronteira Intocável"

A única coisa que impede a migração iterativa de "quebrar o projeto todo de uma vez" é o **Tauri IPC Command Interface**. O Frontend reage e solicita dados estritamente por meio de invocações (Ex: `invoke('get_assets_filtered')`).

A nossa estratégia de segurança garantirá que:
- **As Assinaturas de Entrada e Saída (JSON)** dos Comandos do Tauri e os Eventos que ele emite não mudarão durante a reconstrução do Core.
- O Frontend não saberá que o Backend está usando CQRS ou um Format Registry. Para o React/Solid, o backend continua parecendo "igual", porque a Casca (`delivery/tauri`) blindará a UI da refatoração.

---

## 3. Plano de Ataque em Fases (Solo Dev)

Em vez de Feature Flags complexas rodando duas engines em paralelo, migraremos fisicamente e mentalmente por "Módulos de Domínio", permitindo testes ágeis diretos na tela.

### Fase 1: Reorganização Física (O "Tapa" Visual)
Duração estimada: *Rápida*
- **Ação:** Mover os arquivos existentes de `src-tauri/src/` para a nova taxonomia (`core/`, `feature/`, `processing/`, `infra/`, `delivery/`). 
- **Detalhe:** Nesta fase, _não_ reescrevemos o código. O `thumbnail_worker` antigo apenas muda de diretório para `processing/thumbnails/`. Consertam-se os `mod.rs` e `use paths` até a compilação voltar ao verde (`cargo build`).
- **Validação:** A aplicação rodará exatamente como antes, mas com uma árvore de pastas pronta para escalabilidade.

### Fase 2: O Núcleo Silencioso (Dark Code)
Duração estimada: *Média*
- **Ação:** Criar as entidades estruturais do modelo ideal puras do zero em `core/`.
  - Criar o `AssetLedger` falso/vazio.
  - Implementar o `EventBus` (`tokio::sync::broadcast`) e definir os Enum types dos Eventos (ex: `AssetTagsUpdatedEvent`).
  - Esboçar os *Traits* (Interfaces) do `FormatRegistry`.
- **Validação:** Como os códigos são passivos e apenas testes unitários em Rust locais os enxergarão, zero quebras ocorrem no App rodando.

### Fase 3: Substituição do Cérebro pelo Ledger (Cortesia do CQRS)
Duração estimada: *Pesada*
- **Ação:** Mapear todos os comandos de gravação existentes no Tauri (Ex: Editar, Apagar, Renomear) para o fluxo do `feature/` (Command Handlers) e forçar todos a gravarem através do *AssetLedger* e não mais do SQLx direto.
- As Mutações abandonam o uso direto de SQLite.
- As Queries (Leituras visuais) podem até temporariamente continuar acessando o DB (ou serem movidas para `QueryHandlers`), pois não afetam consistência.
- **Validação:** Após codificado, você pode simplesmente clicar no Frontend para trocar uma Tag. Se no Frontend reagir corretamente é porque o Ledger engoliu a alteração e despachou o evento com maestria.

### Fase 4: Desacoplamento do Watcher e Mídia (Event-Driven)
Duração estimada: *Pesada*
- **Ação:** O File Watcher antigo, que invadia o DB para registrar novas mídias, é castrado. Ele passará a ser apensa um "Sensor" em `processing/watcher/`, que cospe o `FileDiscovered` para o `EventBus`.
- Módulos paralelos (atores/workers no `processing/`) captam esse aviso para rodar os extratores (FFmpeg, Imagens), registrar no DB ou desenhar as Thumbnails.
- **Validação:** Você joga uma imagem em uma pasta lida sob vigilância; se a thumbnail aparecer mágica no Solid.js, a cadeia assíncrona orientada a eventos funcionou com êxito.

---

## 4. Estratégia de Testabilidade (Frontend como Test Driver)

Por ser o único desenvolvedor, você pode se dar ao luxo de deletar o banco `mundam.sqlite` a qualquer momento para validar recriações. 

A "Testabilidade" oficial passará a usar o Design System / App Flow do Front como Árbitro da verdade. 
Sempre que uma Fase (3 ou 4) for implementada, o fluxo de aceitação é empírico:

1. **Drop DB:** Apague o SQLite.
2. **Cold Start:** Inicie a base; o novo EventBus aguenta o Watcher reprocessar tudo sem Panic?
3. **Data Integrity Check:** Clique nos vídeos e veja miniaturas 3D/PDF; o Tauri Request entregou com a mesma performance e sem rasgar o JSON das queries?
4. **Mutational Test:** Tente excluir um lote de 10 fotos no Front. Se nenhum erro estourar no console e a UI atualizar via Evento IPC em tempo real vindo do Rust, o Teste E2E "passou".

---

## 5. Resumo da Abordagem Ágil

Por estarmos num cenário sem bases em Produção Ativa com Clientes, ganhamos alavancagem de reescrita em blocos enormes (`Big Bang` em Módulos, ao invés de no projeto inteiro). O Custo do "erro" será consertar o código imediatamente se a UI quebrar, pois não há dor de *user disruption*. 

Esta cadência assegura que, em questão de poucas semanas, o coração Hexagonal esteja inteiramente implantado, e a complexidade arquitetural extra só brilhará no momento que o MUNDAM despontar na máquina de potenciais novos artistas sem lag e sem arquivos fantasmas.
