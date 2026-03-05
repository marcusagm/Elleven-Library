A consolidação da arquitetura híbrida para o Mundam — unindo a organização orientada a serviços com a resiliência do modelo Hexagonal/EDA/CQRS — é uma decisão excelente. Criar uma documentação densa e bem estruturada antes de codificar garantirá que as peças complexas do Rust (como *borrow checker*, *lifetimes* e *concorrência assíncrona*) não descarrilem o projeto.

Para cobrir todos os aspectos dessa migração robusta, adicionei arquivos estratégicos à sua ideia original. Eles preencherão lacunas sobre persistência de dados, contratos de comunicação com o React e a estratégia de convivência (Strangler Fig).

### Estrutura de Diretórios Sugerida

```text
backend-architecture/
├── definition/
│   ├── overview.md
│   ├── roadmap.md
│   ├── data-model-and-state.md
│   ├── tauri-ipc-contracts.md
│   ├── migration-strategy.md
│   ├── guidelines.md
│   └── modules/
│       ├── 01-asset-ledger-and-cqrs.md
│       ├── 02-event-bus-and-reactivity.md
│       ├── 03-format-kit-registry.md
│       ├── 04-unified-error-handling.md
│       ├── 05-job-scheduler-workers.md
│       ├── 06-fs-watcher-indexer.md
│       └── 07-streaming-delivery.md
└── sprints/
    ├── sprint-01-foundation-and-format-kit.md
    ├── sprint-02-ledger-and-event-bus.md
    ├── sprint-03-read-models-and-queries.md
    ├── sprint-04-workers-and-scheduler.md
    ├── sprint-05-fs-watcher-refactoring.md
    └── sprint-06-legacy-sunset.md

```

---

### Detalhamento dos Arquivos

| Arquivo / Diretório | Propósito e Conteúdo |
| --- | --- |
| **`overview.md`** | Visão macro do sistema. Contém o diagrama Mermaid principal, a explicação da escolha híbrida, a organização das pastas físicas do Rust e um glossário rápido dos termos (ex: Ledger, Command, Port). |
| **`roadmap.md`** | A linha do tempo do projeto. Define os marcos de sucesso, KPIs esperados após a migração e a ordem macro de execução das fases. |
| **`data-model-and-state.md`** | Documenta o esquema atualizado do SQLite, as tabelas de auditoria do CQRS e as máquinas de estado dos assets (ex: `Discovered` -> `Indexed` -> `Thumbnailed`). |
| **`tauri-ipc-contracts.md`** | Define as assinaturas JSON esperadas entre o Frontend (Solid/React) e os `CommandHandlers` / `QueryHandlers` do Backend. Essencial para o desenvolvimento paralelo. |
| **`format-implementation-guide.md`** | Passo a passo direto ao DEV de como implementar extensões, migrar lógicas antigas (ex: extração de PDF/Vídeo) para assinar as `Capabilities` da Arquitetura Hexagonal. |
| **`migration-strategy.md`** | Documenta a estratégia ágil de migração "Agressiva e Direta" (focada em cenário pré-produção com Dev único). Baseia-se em substituição de Módulos (Sem duplicação de V1/V2) e utiliza o Frontend/Tauri IPC como suíte de validação E2E empírica. |
| **`guidelines.md`** | Documenta as diretrizes de desenvolvimento, incluindo padrões de código, convenções de nomenclatura, padrões de design, padrões de teste e padrões de segurança. |
| **`modules/*.md`** | Cada arquivo destrincha um domínio específico. Deve conter: Responsabilidades, Diagrama de Sequência ou Classe (Mermaid), Definição das Traits (Interfaces) e Estratégias de Tratamento de Erro. |
| **`sprints/*.md`** | Guias táticos de execução. Cada arquivo deve conter os objetivos da sprint, os critérios de aceite, os arquivos exatos a serem criados/modificados e a estratégia de testes para aquela etapa. |

---

### Dicas para Desenvolvimento Consistente com Gemini 3.1 Pro

Para extrair o máximo do assistente durante a implementação do código Rust e manter a aderência a essa arquitetura, aplique estas práticas no seu fluxo:

* **Injeção de Contexto Focado:** A janela estendida do Gemini 3.1 Pro permite absorver muitos arquivos, mas a precisão aumenta com o foco. Ao iniciar uma sessão de código, forneça apenas o `overview.md`, o `module.md` específico que será trabalhado e o plano da `sprint-X.md` atual.
* **Aprovação de Plano por Etapas:** Sempre inicie o prompt exigindo que o assistente leia a documentação fornecida e gere um plano de execução dividido em etapas curtas para a tarefa do dia. Isso garante que a abstração Hexagonal não seja atropelada. Aprove ou ajuste o plano antes de permitir a geração do código.
* **Imposição de Código Limpo e Direto:** Reforce em seus prompts a exigência de que explicações sobre as escolhas de arquitetura e refatoração sejam feitas exclusivamente no texto do chat. Lembre o assistente de fornecer o arquivo Rust completo quando a alteração de Traits ou *Lifetimes* for estrutural e complexa, garantindo que você não precise adivinhar onde inserir os blocos.
* **Tratamento de *Borrow Checker* e EDA:** A Arquitetura Orientada a Eventos em Rust pode gerar complexidades com referências (`Arc`, `Mutex`, `tokio::sync`). Cole os logs de erro do compilador na íntegra no chat; o assistente usará o contexto das Traits documentadas nos arquivos `modules/` para resolver o escopo de tempo de vida das variáveis.
* **Validação de *Feature Flags*:** Ao pedir a implementação de um novo fluxo (como os novos extratores de metadados), instrua o assistente a iniciar escrevendo a lógica de chaveamento do roteador V1/V2, garantindo que o legado permaneça intacto conforme planejado na estratégia de migração.
