# Plano de Otimização: Navegação e Atalhos de Teclado (Mundam)

Este plano detalha a reformulação do sistema de entrada para garantir uma navegação fluida, sem conflitos de foco e com suporte robusto a atalhos contextuais.

---

## 1. Diagnóstico de Problemas Atuais

1.  **Conflitos de Foco:** Atalhos globais (como `f` para busca) disparam mesmo quando o usuário está digitando em um campo de texto.
2.  **Falta de Escopos Claros:** Não há uma hierarquia rígida que impeça atalhos de componentes em segundo plano de serem executados.
3.  **Navegação em Grid Inconsistente:** A navegação por setas no grid de assets perde o foco ao carregar novos itens ou ao abrir o inspetor.
4.  **Feedback Visual:** O usuário não tem indicação clara de qual elemento detém o "foco de atalho" atual.

---

## 2. Nova Arquitetura: Sistema de Escopos (Input Scopes)

Implementaremos uma pilha de escopos com prioridades e bloqueio seletivo.

### Hierarquia de Prioridades
| Escopo | Prioridade | Descrição |
| :--- | :--- | :--- |
| `modal` | 100 | Bloqueia todos os atalhos inferiores. |
| `inspector` | 80 | Atalhos específicos para visualização de arquivos. |
| `search` | 60 | Ativo quando a barra de busca está focada. |
| `grid` | 40 | Navegação principal entre arquivos. |
| `global` | 0 | Atalhos básicos (ajuda, configurações). |

### Regras de Bloqueio
*   **`blockLowerScopes: true`**: Quando um escopo (como um Modal) está ativo, ele impede que qualquer atalho de prioridade menor seja processado.
*   **Detecção de Input Inteligente**: Atalhos de tecla única (sem modificadores como Ctrl/Alt) serão **automaticamente bloqueados** se um elemento `input` ou `textarea` estiver focado, a menos que o atalho pertença explicitamente ao escopo do input.

---

## 3. Etapas de Implementação

### Fase 1: Refatoração do Dispatcher
*   **Ação:** Atualizar `isInputFocused` em `dispatcher.ts` para ser mais rigoroso.
*   **Melhoria:** Modificar a lógica de filtragem para respeitar a flag `blockLowerScopes` da pilha de escopos do `inputStore`.

### Fase 2: Hook `useInputScope`
*   **Ação:** Criar um hook SolidJS para gerenciar o ciclo de vida dos escopos.
*   **Exemplo de Uso:**
    ```typescript
    useInputScope({
      name: 'search',
      priority: 60,
      blockLowerScopes: false // Permite atalhos globais como Ctrl+S
    });
    ```

### Fase 3: Padronização de Atalhos
*   **Navegação:** `Arrows` para mover, `Enter` para abrir, `Space` para seleção rápida.
*   **Busca:** `/` ou `f` para focar busca, `Esc` para limpar/desfocar.
*   **Inspetor:** `[` e `]` para navegar entre arquivos enquanto o inspetor está aberto.

### Fase 4: Acessibilidade e Feedback
*   **Indicador de Foco:** Adicionar um contorno visual (ring) distinto para o elemento que possui o foco de teclado atual.
*   **Guia de Atalhos:** Atualizar o painel de ajuda (`?`) para mostrar atalhos dinamicamente baseados no escopo ativo.

---

## 4. Cronograma Estimado

| Atividade | Esforço | Prioridade |
| :--- | :--- | :--- |
| Refatoração do Dispatcher e Store | 1 dia | Crítica |
| Implementação do Hook `useInputScope` | 1 dia | Alta |
| Correção da Navegação em Grid | 2 dias | Alta |
| Testes de Conflito (Input vs Global) | 1 dia | Crítica |

---

## 5. Próximos Passos Recomendados
1.  Substituir o uso direto de `pushScope` nos componentes pelo novo hook `useInputScope`.
2.  Implementar o bloqueio automático de teclas alfanuméricas simples em campos de texto no nível do `KeyboardProvider`.
