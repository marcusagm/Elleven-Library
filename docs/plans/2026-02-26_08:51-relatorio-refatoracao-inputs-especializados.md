# Relatório de Refatoração: Inputs Especializados

**Data:** 26 de Fevereiro de 2026  
**Status:** ✅ Concluído  
**Assunto:** Reestruturação arquitetural e padronização de `NumberInput`, `MaskedInput` e `TagInput`.

---

## 📖 Contexto e Objetivos

O objetivo desta tarefa foi modernizar os componentes de input especializados, movendo-os de uma implementação de arquivo único para uma estrutura de pastas atômica, seguindo o padrão já estabelecido para componentes como `ColorInput` e `Input`.

As principais metas foram:
1.  **Conformidade com SOLID:** Divisão de responsabilidades, especialmente no complexo `TagInput`.
2.  **Padronização de Nomenclatura:** Eliminação total de abreviações conforme `frontend-solid.md`.
3.  **Documentação Exaustiva:** Adição de **TSDoc** em todas as interfaces e componentes.
4.  **Segurança de Input:** Integração com o sistema core de shortcuts (`useInputEvents`).

---

## 🛠️ Passo a Passo da Implementação

### 1. Reestruturação de Pastas
Foram criados diretórios individuais para cada componente em `src/components/ui/`, permitindo a colocalização de tipos, sub-componentes e estilos específicos.

### 2. Refatoração do `NumberInput`
-   **CSS:** Arquivo movido e classes renomeadas de `.ui-number-input-btn` para `.ui-number-input-button`.
-   **Lógica:** Implementação utilizando `createControllableSignal` para suportar estados controlados e não controlados.
-   **Componente Base:** Agora utiliza o componente `Input` interno, herdando automaticamente o tratamento de atalhos de teclado.

### 3. Refatoração do `MaskedInput`
-   **Interface:** Definição clara da prop `mask` com documentação sobre o padrão suportado (atualmente apenas '9' para dígitos).
-   **Responsabilidade:** Foco exclusivo na aplicação da máscara via regex, delegando a renderização ao componente `Input`.

### 4. Decomposição Atômica do `TagInput`
Este foi o componente mais transformado devido à sua alta complexidade.
-   **Root (`TagInput.tsx`):** Gerencia apenas o estado da lista de tags, filtragem de sugestões e posicionamento do portal.
-   **`TagChip.tsx`:** Sub-componente isolado para a renderização visual da tag e seu botão de remoção.
-   **`TagSuggestions.tsx`:** Gerencia a exibição da lista de autocompletar em um `Portal`, garantindo que o dropdown não seja cortado por overflows de containers pais.

### 5. Limpeza e Integração
-   Remoção dos arquivos antigos em `src/components/ui/`.
-   Atualização do `index.ts` central de UI para expor os novos componentes através de globais (`export *`).

---

## 🚧 Obstáculos e Soluções

| Obstáculo | Solução |
| :--- | :--- |
| **Avisos de Reatividade:** O SolidJS emitia avisos ao passar variáveis reativas diretamente para o hook `useInputEvents`. | Foram feitos ajustes para garantir que propriedades reativas sejam acessadas apenas dentro de contextos rastreados ou via `mergeProps`/callbacks. |
| **Tipagem de Refs:** Incompatibilidade entre `HTMLElement` e `HTMLUListElement` ao disparar lógica de `clickOutside`. | Refatoração das interfaces para usar tipos específicos de elementos HTML, garantindo robustez no TypeScript. |
| **Shortcut Safety:** `TagInput` precisava bloquear atalhos globais (como 'Delete' ou 'Espaço') sem quebrar sua própria lógica de remoção de chips. | Integração manual do `handleKeyDown` do core com a lógica local de navegação de sugestões. |

---

## 🚀 Melhorias Futuras

1.  **Hook de Posicionamento:** Extrair a lógica de cálculo de posição do `TagInput` para um hook genérico `useOverlayPositioning`.
2.  **Máscaras Avançadas:** Expandir o `MaskedInput` para suportar padrões alfanuméricos complexos e caracteres opcionais.
3.  **Acessibilidade (Aria-Live):** Adicionar anúncios de `aria-live` quando tags são adicionadas ou removidas para melhor suporte a leitores de tela.
4.  **Testes Unitários:** Implementar testes específicos para a lógica de incremento/decremento do `NumberInput` e filtragem de tags no `TagInput`.
