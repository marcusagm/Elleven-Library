# 📚 Refatoração e Padronização de Atalhos UI (UI Navigation Shortcuts)

**Data:** 2026-03-03
**Área:** Arquitetura de Frontend / UI Components / Input System

## 🎯 Objetivo
O objetivo principal dessa intervenção foi despoluir o painel de configurações de atalhos do Mundam (`KeyboardShortcutsPanel.tsx`). O painel estava exibindo, e permitindo a edição, de dezenas de atalhos internos e temporários de usabilidade de componentes específicos (ex: Modal, Dropdown, Slider, TagInput e TreeView). Esses atalhos (como 'Escape' para fechar um modal ou setas direcionais navegar em um Dropdown) são atalhos de navegação do DOM baseados em foco, que seguem padrões da W3C / ARIA e não deveriam ser expostos como macro-comandos da aplicação no painel de preferências do usuário.

Em paralelo, identificamos e corrigimos um problema grave de acessibilidade no qual o usuário perdia a semântica nativa da tecla "Enter" e "Space" (usada para clicar no `<button>` nativo) quando esse botão estava dentro de uma camada de navegação gerenciada (Viewport), pois o atalho global de "Open Item" (`Enter` no viewport) possuía prioridade máxima consumindo o evento da tela.

---

## 🛠️ O Que Foi Feito (Passo a Passo)

### 1. Extensão da Interface `ShortcutDefinition` (`types.ts`)
- Mapeada e adicionada a diretiva opcional `system?: boolean` tanto na interface `ShortcutDefinition` (tempo de execução) quanto em `SerializedShortcut` (persistência).  
- **Motivo:** Flag para assinalar a nível arquitetural quando um atalho serve apenas a um bloco semântico padrão (internal UI wiring), diferenciando atalhos nativos de tela de "Macro Ações" (como *Focus Search* ou *Select All*).

### 2. Integração no Primitivo `createShortcut` (`createShortcut.ts`)
- O hook primário utilizado para construir comandos agora reconhece `options.system` em seu payload.
- **Motivo:** Quando o `shortcutStore.register(...)` insere esse item em memória, ele avisa permanentemente o store sobre o tipo do atalho em questão.

### 3. Limpeza UI do Settings Panel (`KeyboardShortcutsPanel.tsx`)
- O iterador e memorizador de atalhos passou a filtrar qualquer atalho com flag `system === true`.
```tsx
const groupedShortcuts = createMemo((): ScopeGroup[] => {
    // Filter out internal system shortcuts
    const shortcuts = shortcutStore.list().filter(s => !s.system);
    // ...
});
```
- **Motivo:** O painel de Configurações do Mundam volta a ser um painel pragmático com comandos verdadeiramente de escopo de aplicativo.

### 4. Correção do Conflito de Foco / Acessibilidade (Enter no Viewport) (`dispatcher.ts`)
- Em `dispatcher.ts`, havia uma constatação `isInputFocused()` que avaliava o comportamento e abortava atalhos apenas com flag `ignoreInputs`, que se limitava a `<input>` ou `<textarea>`.
- Criamos a função `isInteractiveFocused(target)` que agora não observa os *Text Inputs*, mas sim toda a gama de elementos de UI interativos: `<button>`, `<a>`, `<select>`, `<summary>`, e elementos que utilizam o atributo WAI-ARIA `role="button/link/tab/radio/etc"`.
- Alterada a subrotina para invocar os parâmetros através de `shouldYieldToInteractive()`, resolvendo `complexidades ESLint` e garantindo que o escopo global do dispatcher "retraia" (retornando `false`) quando encontrar uma macro como "Space" ou "Enter", se o DOM ativamente apontar para a execução da tarefa interativa original do Browser.
- **Motivo:** Essa foi e a resolução "Opção D". Nós mantemos a robustez do `blockLowerScopes` ativando eventos nativos seguros sobre botões focados via *Tab*. 

### 5. Configuração Extensa nos Componentes de UI 
- Atualizados em toda a interface base as injeções `{ system: true }` das chaves gerenciadas de:
    - `src/components/ui/Modal/ModalContent.tsx`
    - `src/components/ui/Slider/SliderThumb.tsx`
    - `src/components/ui/DropdownMenu/useMenuNavigation.ts`
    - `src/components/ui/TreeView/hooks/useTreeNavigation.ts`
    - `src/components/ui/TagInput/hooks/useTagNavigation.ts`

---

## 🚧 Obstáculos e Regressões Possíveis
Durante a resolução foram avaliados três possíveis modelos de correção: a adoção em massa de manipuladores `onKeyDown` perdendo os controles de "Scopes", a migração para Handlers Macros ou um esquema centralizado usando os eventos interativos. 
A arquitetura `Opção D` se demonstrou o sweet-spot de resolução preservando os pontos fortes do framework (Escopo Condicional `createConditionalScope`) em detrimento do risco. O maior alerta para regressão em desenvolvimentos futuros estaria concentrado no array de tags HTML mapeadas em `INTERACTIVE_ELEMENTS`.

## 🔮 Sugestões de Melhorias Futuras
1. Em `inputStore.ts` pode se construir uma ferramenta de dev overlay / console local, atrelada à flag de DEBUG que possa iterar atalhos mesmo em background. Como agora filtramos os resultados "system" globalmente na UI, uma tela administrativa facilitaria visualizações perante hooks isolados de teclado.
2. Migrar, lentamente as submissões visuais nativas de UI dos Forms (Enter envia form) a usarem uma hierarquia semelhante se demonstrarem ser instáveis localmente na árvore.
