# Guia de Execução de Sprint com Agentes de IA

Este documento é a **Bíblia Operacional** a ser seguida sempre que você (Usuário) for iniciar uma nova Sprint documentada no `roadmap.md` ao lado de um Agente de IA (como eu). 

O objetivo supremo deste fluxo é garantir que **a transição arquitetural ocorra com Zero Downtime (sem quebrar os recursos legados atuais)** e extraindo códigos limpos, blindados pelo compilador do Rust, e que respeitem estritamente as regras de Clean Code e CQRS.

---

## 🛑 Regra de Ouro: O Padrão Strangler Fig (V1 vs V2)
Até que a **Fase 5** seja totalmente concluída e o Frontend aponte 100% para os Novos *Command Handlers*, o código antigo legado (`src-tauri/src/v1_legacy` ou arquivos originais não refatorados na raiz) **NÃO PODE SER APAGADO**. 

*   O Agente irá construir as novas estruturas nas pastas `core/`, `feature/`, `infra/` e `delivery/` **paralelamente** ao que já existe.
*   Feature Flags ou rotas espelho (ex: um Tauri Command extra `invoke('v2_get_assets')`) podem ser usadas temporariamente em tela de desenvolvedor para garantir que o Front end real continue intacto consumindo a interface V1 antiga.
*   Deleção de código velho (Legacy Sunset) só ocorrerá explicitamente no fim do projeto.

---

## 🛠️ Passo a Passo para Iniciar uma Sprint

### Passo 1: O "Setup" do Prompt de Contexto Mínimo
Não jogue todo o repositório de uma vez para o Agente. O excesso de escopo corrompe a qualidade do código gerado (redução de precisão). Ao abrir o Chat/Sessão para trabalhar na `Sprint X`, anexe **apenas** estes documentos essenciais:

1.  `guidelines.md` (Para o agente absorver os padrões de Código, SRP, Tratamento de erros e Documentação Rustdoc exigidos).
2.  `overview.md` (Para o agente lembrar que o projeto usa CQR e Arquitetura Hexagonal).
3.  O `modules/XXX.md` relevante àquela Sprint (Ex: Se for a Sprint 3.1, envie `03-format-kit-registry.md`).
4.  O arquivo da Sprint escolhida (`sprints/sprint-X-X.md`).

**Exemplo de Prompt Inicial:**
> *"Agente, hoje vamos iniciar a execução da **Sprint 1.1**. Li e anexei o manifesto da Sprint, as Guidelines do projeto e o Overview Arquitetural. Não gere nenhum código ainda. Analise estes documentos e crie um plano de passos (Checklist de arquivos a serem criados/editados) para atingirmos os Critérios de Aceite."*

### Passo 2: A Validação do Plano (Hold & Approve)
O Agente responderá com um roteiro. Cabe a você ler com olhar clínico:
*   Ele propôs violar o isolamento? (Ex: Pediu pra pôr query SQL no `core/`)?
*   Ele sugeriu deletar recursos que a atual aplicação em V1 utiliza?
*   Se o plano não for perfeito, exija correções e direcione-o. Mande-o focar estritamente nos Princípios da Arquitetura Hexagonal.

### Passo 3: Geração Gradual (Isolamento de Erros)
Com o plano aprovado, instrua o Agente a codificar **um arquivo/módulo por vez**.
1.  **Peça a Criação:** "Execute o Passo 1 do plano estabelecido (Criar structs de Command)."
2.  **Verifique a Qualidade:** O agente usou `unwrap()`? Faltaram as `///` do Rustdoc? Exija a correção na hora.
3.  **Trave o Compilador:** Tente rodar o projeto localmente (`cargo check`). Erros de *Lifetimes/Borrow Checker* surgirão. **Copie o erro do compilador na íntegra** e cole no chat para que o Agente corrija antes de avançar para a próxima etapa.

### Passo 4: O Teste de Aceite Intermediário
Antes de fechar a Sprint, você precisará provar que o código gerado faz sentido E2E.
*   Siga as instruções estritas da área **"Critérios de Aceite"** de cada *Sprint Document*.
*   Exija que o agente crie `#[tokio::test]` robustos acoplados no pé do arquivo para testarmos as mecânicas.
*   Pelo menos um Command Tauri provisório (Mock Route) deve ser configurado para você clicar via React/SolidJS e confirmar a ida e volta correta (comprovando os parsings de JSON sem crash).

### Passo 5: Atualização de Status
Após a consolidação tática (Os testes passaram e a feature isolada V2 respira paralelamente):
1. Abra o arquivo físico da sprint (`sprints/sprint-X-X.md`).
2. Mude seu cabeçalho para `**Status:** Concluído`. Preencha a Data de Fim.
3. Commit! E passe (num chat novo limpo) para a próxima Sprint contígua.

---

## 🧠 Dicas Essenciais para Comportamento do Agente

Quando estiver "dialogando" com o código, reforce as regras base sempre que o agente tentar ser preguiçoso:

-   **"Sem explicações longas sobre o código."** O código já deve se auto-explicar através de boas nomenclaturas e uso intensivo dos Padrões Rustdoc `///`. Se algo for muito complexo, exija um comentário atrelado na linha do Rust, não no corpo do Chat de IA.
-   **"Devolva a estrutura completa."** Às vezes o agente retorna blocos resumidos `// ... código existente`. Peça explicitamente: *"Devolva o arquivo X por completo para evitar erros de merge no parser"*.
-   **"Respeite os Handlers de Erro."** Constantemente lembre o Agente que ele DEVE empacotar qualquer `result::Err` na struct universal `AppError` formulada lá no princípio do roadmap (Fase 1). 
