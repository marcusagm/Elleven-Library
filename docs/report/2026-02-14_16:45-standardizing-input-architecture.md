# Relatório de Análise e Otimização do Sistema de Input e Acessibilidade

**Data:** 14 de Fevereiro de 2026  
**Status:** Em Progresso  
**Contexto:** Análise da arquitetura de atalhos, foco e input (Global, Search, ItemView, Modal) para resolver problemas de acessibilidade (ex: ESC não funcionando) e padronizar o sistema.

---

## 1. Análise da Arquitetura Atual

O sistema de input do Mundam é robusto e segue um padrão de "Stack de Escopos" com prioridades. Arquivos principais:
- **`dispatcher.ts`**: O núcleo que processa eventos de teclado e decide qual atalho disparar baseado na pilha de escopos ativos.
- **`inputStore.ts`**: Mantém o estado global (teclas pressionadas, pilha de escopos).
- **`shortcutStore.ts`**: Registro central de definições de atalhos (nome, teclas, comando, escopo).
- **`KeyboardProvider`**: Ouve eventos do DOM (`document.keydown`) e normaliza para tokens.

### 1.1 Hierarquia de Escopos (Stack Logic)
A prioridade define quem "vence" conflitos e quem pode bloquear escopos inferiores.
*Prioridades (definidas em `types.ts`):*
1.  **Modal** (`priority: 1200`) - Bloqueia tudo abaixo.
2.  **Search** (`priority: 1100`) - Bloqueia navegação e edição.
3.  **Editing** (`priority: 1000`) - Bloqueia visualizadores.
4.  **Image Viewer** (`priority: 50`) - Bloqueia Viewport (grid).
5.  **Viewport** (`priority: 10`) - Navegação principal.
6.  **Global** (`priority: 0`) - Atalhos gerais (Settings, Select All).

### 1.2 O Problema do ESC no ItemView
Atualmente, o ESC não fecha o ItemView.
**Diagnóstico Provável:**
1.  **Lógica de Bloqueio (`blocking`)**: O `ItemView` ativa o escopo `image-viewer` com `blocking: true`. Isso impede que atalhos de escopos menores (Global: priority 0) funcionem.
2.  **Definição do Atalho**: Embora o ESC esteja definido localmente no `ItemView` (via `useShortcuts`), ele depende do comando `viewer:close`.
3.  **Conflito de ID/Comando**: Se o atalho Global "Deselect All" (ESC) também estiver registrado, e o sistema de bloqueio impedir o Global, o `ItemView` deveria pegar. Contudo, se houver qualquer falha na ativação do escopo `image-viewer` ou se a prioridade calculada estiver equivocada, o evento é descartado.
4.  **Possível Falha de Foco**: O `ItemView` foca um `div` (overlay). Se o dispatcher considerar isso como "Input" (improvável, pois é div), poderia ignorar. Mas o mais provável é que a definição do atalho no `shortcutStore` precise ser explícita e única para o escopo.

---

## 2. Inventário de Componentes e Inputs

### 2.1 Global
- **Atalhos**: Select All (Ctrl+A), Settings (Ctrl+,), Focus Search (Ctrl+K).
- **Tratamento**: Sempre ativo (priority 0).

### 2.2 Search (Busca)
- **Componente**: `SearchBox` / `SearchModal`.
- **Atalhos**: Clean/Close (ESC), Navigate results (Up/Down), Open (Enter).
- **Tratamento**: Deve isolar input enquanto digita (evitar disparar atalhos globais de letras).

### 2.3 ItemView (Visualizador)
- **Componente**: `ItemView`.
- **Atalhos Completos Necessários**:
    -   `ESC`: Fechar.
    -   `Left/Right`: Anterior/Próximo.
    -   `Space`: Play/Pause Slideshow ou Pan/Select.
    -   `+ / - / =`: Zoom.
    -   `0 / 1`: Fit / Original size.
    -   `R`: Rotate, `H`: Pan, `V`: Flip Vertical, `Shift+H`: Flip Horizontal.

### 2.4 Modal
- **Atalhos**: Confirm (Enter), Close (ESC).
- **Tratamento**: Trap focus obrigatório. Bloqueio total de escopos inferiores.

### 2.5 Lista de Arquivos (Viewport)
- **Componente**: `VirtualMasonry` / `VirtualGrid` / `VirtualList`.
- **Atalhos**: Setas (Navegação), Home/End, PageUp/PageDown, Enter (Abrir), Space (Selecionar).

---

## 3. Plano de Otimização e Refatoração

Para tornar a usabilidade "perfeita e customizável", faremos as seguintes alterações estruturais:

### Passo 1: Centralização Absoluta no `shortcutStore`
Atualmente, componentes usam `useShortcuts` definindo teclas hardcoded.
- **Mudança**: O `shortcutStore` conterá **TODOS** os atalhos padrão do sistema.
- **Benefício**: Usuário pode customizar *qualquer* tecla nas configurações.
- **Ação**: Mover definições do `ItemView`, `Viewport` e `Search` para `DEFAULT_SHORTCUTS` no store.

### Passo 2: Componentes "Consumers" (Pattern `useCommand`)
Em vez de redefinir atalhos, os componentes apenas "assinam" comandos.
- Criar hook `useCommand(commandId, handler)`.
- O componente não sabe qual tecla dispara `viewer:close`, apenas sabe que deve fechar quando o comando ocorrer.
- `useShortcuts` deve ser usado apenas para atalhos *efêmeros* ou muito específicos que não merecem estar nas configurações globais.

### Passo 3: Refinamento da Hierarquia e Bloqueio
- Garantir que `ESC` tenha definições explícitas em cada escopo (`global`, `modal`, `image-viewer`, `search`).
- O `dispatcher` deve garantir que o `ESC` do escopo *mais alto* vença, mesmo que todos usem a mesma tecla.

### Passo 4: Feedback Visual e Acessibilidade
- Adicionar sons (opcionais) ou feedback visual ao acionar atalhos.
- Garantir que `FocusTrap` funcione corretamente em Modais e ItemView para leitores de tela.

---

## 4. Implementação Detalhada (Passo a Passo)

### 4.1 Atualizar `types.ts` e `shortcutStore.ts`
Adicionar todos os atalhos de navegação, visualização e edição na lista `DEFAULT_SHORTCUTS`.

### 4.2 Criar Hook `useCommand`
Abstração para ouvir eventos do dispatcher sem registrar novas teclas.

### 4.3 Refatorar `ItemView.tsx`
Remover `useShortcuts` com definições de teclas. Substituir por `useCommand` ou `useShortcuts` referenciando apenas IDs/Comandos já existentes, sem hardcode de teclas se possível (ou mantendo apenas como fallback).

### 4.4 Refatorar `dispatcher.ts` (Se necessário)
Verificar lógica de `blockLowerScopes`. Se `Image Viewer` (50) bloqueia `Global` (0), mas o usuário aperta `ESC` (definido em ambos), o dispatcher DEVE disparar o do `Image Viewer`.

---

## 5. Melhorias Futuras
- **Editor de Atalhos (UI)**: Permitir que usuário filtre por escopo.
- **Sequências (Leader Keys)**: Suporte a `G G` (Go Global) ou `Space f` (Find). Já suportado pelo engine, precisa de UI.
- **Export/Import**: Salvar configurações de teclado em JSON.
