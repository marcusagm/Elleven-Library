# Refatoração DropdownMenu e ContextMenu

**Data:** 2026-02-25
**Status:** Concluído
**Autor:** Antigravity

## Objetivo
Modularizar os componentes `DropdownMenu` e `ContextMenu`, eliminando dívidas técnicas de tipagem (`any`), melhorando a acessibilidade via teclado e alinhando os atalhos com o sistema central de inputs da aplicação.

## Etapas de Implementação

### 1. Reestruturação de Pastas
- Movidos componentes monolíticos `src/components/ui/DropdownMenu.tsx` e `src/components/ui/ContextMenu.tsx` para pastas dedicadas.
- **Estrutura criada:**
  - `components/`: Sub-componentes internos (`MenuList`, `MenuItem`, `MenuStateItems`).
  - `hooks/`: Lógica de posicionamento (`useMenuPositioning`) e navegação (`useMenuNavigation`).
  - `types.ts`: Definições estritas de tipagem.
  - `index.ts`: Ponto de entrada limpo.

### 2. Tipagem Estrita (Zero `any`)
- Implementação de **Discriminated Unions** para os itens de menu:
  - `ActionMenuItem`, `CheckboxMenuItem`, `RadioMenuItem`, `SubmenuMenuItem`, `LabelMenuItem`, `SeparatorMenuItem`.
- Isso permitiu que o TypeScript validasse propriedades específicas (como `checked`, `onCheckedChange` ou `items` recursivos) sem necessidade de casts manuais.

### 3. Alinhamento com `src/core/input`
- Refatoração do hook `useMenuNavigation` para utilizar o sistema central de atalhos.
- **Escopo `menu`:** Criado um escopo de input dinâmico que é ativado quando o menu abre, bloqueando atalhos globais conflitantes.
- **Shortcuts registrados via `createShortcut`:**
  - `ArrowUp`/`ArrowDown`: Navegação entre itens.
  - `Home`/`End`: Pulo para início/fim.
  - `Enter`/`Space`: Execução de ação ou toggle de checkbox.
  - `Escape`/`Tab`: Fechamento do menu.

### 4. Melhorias de Posicionamento e UX
- Integração robusta com `@floating-ui/dom` no `DropdownMenu` para lidar com colisões nas bordas da tela, flipping e offset.
- Correção no `FolderTreeSidebarPanel` para garantir que o `event.preventDefault()` seja chamado no `onContextMenu`, evitando a abertura simultânea do menu do navegador.

### 5. Correções de Lint e Qualidade
- Ajuste de caminhos relativos para `lib/utils` (profundidade de 4 níveis).
- Remoção de aspas em chaves de objetos para seguir as regras de estilo do projeto.
- Redução de complexidade em funções críticas.

## Obstáculos Encontrados
- **Profundidade de Pastas:** Erros de importação inicial ao mover arquivos para subfolders (necessário ajustar de `../../../` para `../../../../`).
- **Reatividade SolidJS:** Necessidade de garantir que o `onCleanup` dos escopos de input fosse executado corretamente para não "travar" a aplicação no modo menu após o fechamento.

## Possíveis Melhorias Futuras
- **Auto-open Submenus:** Implementar timer para abertura automática de submenus ao passar o mouse.
- **Touch Support:** Adicionar suporte a gestos de swipe para fechar menus em dispositivos móveis.
- **Virtualização:** Suporte a menus extremamente longos (ex: lista de centenas de tags) via virtualização de lista.
- **Animações:** Adicionar micro-interações de fade/scale usando `Solid-Transition-Group`.
