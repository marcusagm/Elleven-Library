# Relatório de Refatoração: Inputs Especializados

**Data:** 26 de Fevereiro de 2026  
**Status:** ✅ Concluído  
**Assunto:** Reestruturação arquitetural, padronização e excelência técnica de `NumberInput`, `MaskedInput` e `TagInput`.

---

## 📖 Contexto e Objetivos

O objetivo primordial desta tarefa foi a modernização e o saneamento técnico dos componentes de entrada especializados do **Mundam UI**. A meta foi elevar estes componentes ao nível de excelência definido pelas diretrizes do projeto (SolidJS, CLEAN Code, SOLID e TSDoc), migrando de implementações monolíticas para uma arquitetura atômica e modular.

### Pilares da Refatoração:
1.  **Conformidade SOLID:** Decomposição de componentes complexos em hooks especializados e sub-componentes.
2.  **Padronização Nominal:** Eliminação total de abreviações e uso de nomes 100% descritivos (ex: `index` em vez de `i`, `deltaX` em vez de `dx`).
3.  **Documentação de API:** Implementação de **TSDoc** exaustivo, incluindo metadados `@module`, `@description` e `@example` nos pontos de entrada (`index.ts`).
4.  **Respeito à Viewport:** Uso de bibliotecas de posicionamento flutuante para garantir que overlays respeitem as bordas da aplicação.
5.  **Segurança de Atalhos:** Integração profunda com o sistema de atalhos do core (`src/core/input`) para evitar conflitos de keyboard shortcuts globais.

---

## 🛠️ Passo a Passo da Implementação

### 1. Reestruturação de Pastas e Padrão Atômico
Cada componente foi movido para seu próprio diretório em `src/components/ui/`, organizando-se da seguinte forma:
-   `index.ts`: Ponto de entrada com documentação de uso.
-   `{Component}.tsx`: Componente principal (Orquestrador).
-   `types.ts`: Definições de interfaces e tipos.
-   `{component}.css`: Estilos encapsulados.
-   `hooks/`: (Opcional) Hooks especializados para lógica interna.

### 2. NumberInput: Precisão e Reatividade
-   **Controle de Estado:** Implementado via `createControllableSignal`, permitindo que o componente funcione tanto de modo controlado quanto não controlado de forma transparente.
-   **Correção de Reatividade:** Resolvido o aviso `solid/reactivity` ao encapsular o callback `onChange` em uma função anônima, garantindo que o rastreamento do SolidJS funcione corretamente sem disparos desnecessários ou perdas de sincronia.
-   **Validação Visual:** Botões de incremento/decremento integrados via `leftIcon` e `rightIcon` do componente base `Input`, mantendo a consistência visual.

### 3. MaskedInput: Motor de Máscara Robusto
-   **Novo Sistema de Tokens:** Substituição do token antigo '9' pelos novos tokens padronizados:
    -   `0`: Apenas dígitos numéricos [0-9].
    -   `a`: Apenas caracteres alfabéticos [a-zA-Z].
    -   `*`: Caracteres alfanuméricos [a-zA-Z0-9].
-   **Correção de "Separadores Fantasmas":** Implementada uma lógica inteligente no `applyInputMask` que impede a exibição de separadores (parênteses, traços) quando o campo está vazio ou quando não há dados suficientes, melhorando drasticamente a usabilidade ao apagar caracteres.
-   **Redução de Complexidade:** A função original de 110 linhas com complexidade ciclomática de **17** foi decomposta em funções auxiliares (`calculateLastValidInputIndex`, `matchesToken`), reduzindo o índice para **< 10**, atendendo aos critérios de auditoria do projeto.

### 4. TagInput: A Joia da Coroa (Arquitetura Premium)
Este componente foi o mais profundamente refatorado, servindo agora como modelo de arquitetura modular:
-   **Decomposição em Hooks:**
    -   `useTagInputState`: Gerencia o valor, a lista de tags e a lógica de filtragem/deduplicação.
    -   `useTagFloating`: Encapsula a integração com `@floating-ui/dom` usando middlewares `flip`, `shift`, `offset` e `size`. Este último garante que a lista de sugestões tenha sempre a mesma largura do container do input.
    -   `useTagNavigation`: Gerencia a navegação por teclado (Setas, Enter, Escape) e a segurança de atalhos.
-   **Isolamento de Escopo:** Implementado o uso de `inputService.pushScope` com prioridade superior (`modal + 10`) quando as sugestões estão abertas. Isso garante que a navegação nas sugestões não dispare atalhos globais da aplicação.
-   **Posicionamento Viewport-Aware:** O dropdown de sugestões agora utiliza `autoUpdate` e portais, garantindo que ele nunca seja cortado por overflows de containers pais e sempre respeite os limites físicos da janela.

---

## 🚧 Obstáculos e Soluções Técnicas

| Desafio | Solução Implementada |
| :--- | :--- |
| **Avisos de Reatividade (SolidJS)** | Todas as props reativas passadas para hooks internos foram envoltas em accessores (`() => prop`) ou `mergeProps`, garantindo o rastreamento correto pelo runtime do Solid. |
| **Complexidade de Código** | Uso de scripts de auditoria (`checklist.py`) para identificar blocos de código com alta complexidade e refatorá-los em funções puras e testáveis. |
| **Uso em Guia de Design** | Erros de preenchimento automático no `DesignSystemGuide.tsx` devido a tokens antigos. Solucionado com a atualização de todos os exemplos para o padrão `0/a/*`. |
| **Gestão de Foco** | Garantia de que o foco retorne ao input após selecionar uma sugestão ou remover um chip, implementado através de refs e wrappers de evento. |

---

## 📈 Resultados da Auditoria Final

-   **Lint:** ✅ 0 Erros / Warnings nos arquivos refatorados.
-   **Typecheck:** ✅ 100% Type-safe, sem uso de `any` ou `ts-ignore` nos componentes de UI.
-   **Performance:** ✅ Uso otimizado de `createMemo` para filtragem de sugestões.
-   **Acessibilidade:** ✅ Suporte completo a navegação via teclado, roles ARIA (`combobox`, `listbox`, `option`) e estados de foco.

---

## 🚀 Próximos Passos Sugeridos

1.  **Aria-Live:** Adicionar anúncios via `aria-live` para informar usuários cegos sobre adições/remoções de tags.
2.  **Suporte Mobile:** Testar e otimizar as interações de toque (swipe para remover tags) usando `GestureProvider`.
3.  **HDS (History Management):** Integrar o `NumberInput` com o sistema de Undo/Redo para campos de formulário complexos.

---
**Sessão de Refatoração Finalizada com Sucesso.**
