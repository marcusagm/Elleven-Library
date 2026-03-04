# 📝 Refatoração e Otimização de Espaço da ListViewToolbar

**Data:** 04 de Março de 2026
**Objetivo:** Melhorar a distribuição de espaço horizontal na barra de ferramentas principal (`ListViewToolbar`), agrupando controles excessivos de layout e ordenação em `Popovers` limpos e padronizados, além de refinar a acessibilidade e feedbacks visuais substituindo atributos `title` nativos por componentes `<Tooltip>`.

---

## 🛠️ Passo a Passo da Implementação

### 1. Re-arquitetura Visual (Adoção de Popovers Pop-up)
A barra original sofria com o "engolimento" da barra de busca (`SearchToolbar`), dadas as quatro fontes independentes de ações (Navegação, Busca, Ordenamento e Filtros Visuais/Tamanho).
- **Ação:** Removemos o componente `DropdownMenu` monolítico e os ToggleGroups/Sliders soltos na raiz.
- **Implementação:** Agrupamos os controles lógicos em dois novos painéis flutuantes (Option B do Brainstorming):
  - **Sort Configuration Popover:** Acessível via um botão `Sort: <Property>`, abrigando agora um `<Select>` para a propriedade de ordenação primária (Title, Date, Rating, etc) e um `<ToggleGroup>` de direção (Ascendente/Descendente).
  - **View Configuration Popover:** Acessível via um botão com ícone de `LayoutGrid`, abrigando opções de exibição de lista (Masonry, Grid, List) e o `<Slider>` de Zoom das miniaturas (reduzindo a ocupação no eixo-X da tela incrivelmente).

### 2. Tratamento Rigoroso de CSS e Nomenclaturas
- **Ação:** As guidelines do `frontend-solid.md` proíbem o uso de abreviações e exigem nomes claros e baseados nos tokens de design do app.
- **Implementação:** Criamos classes puramente semânticas no arquivo `list-view-toolbar.css` como `.toolbar-popover-sort-configuration` e `.toolbar-popover-view-configuration` para controlar a largura, altura livre, gaps e espaçamentos internos de forma fluida nos Popovers refatorados, em oposição à propostas abreviadas.

### 3. Adoção Global do Componente `<Tooltip>`
- **Ação:** Remoção da dependência das pop-ups web nativas geradas pela injeção da tag genérica HTML `title="..."` em botões de ação e modais secundários.
- **Implementação:** Os botões de atalho (*Back/Forward*), opções interativas da pesquisa (*Filtro Ativo, Funnel, Fuzzy e Sliders de Configurações avançadas*) bem como a seleção exata dos layouts agora encontram-se abraçadas pelo componente próprio e gerenciável `Tooltip`, setado via `content` e `placement` adequado.

---

## 🛑 Obstáculos Enfrentados e Soluções

### O Conflito do Elemento <Select> dentro do <Popover>
**O Problema**:
Ao abrir o Menu de Ordenação (`Sort Popover`) e interagir com seu descendente, o `<Select>`, uma anomalia ocorria. Tentar selecionar uma nova opção (`Modified_Date` por exemplo) fechava imediatamente todas as _layers_ ativas (tanto o Dropdown quanto o Menu do Popover raiz), sem nem registrar a nova variável no estado.

**A Causa (Root-Cause)**:
Ao usar componentes de "Portal", como o `<SelectContent>`, a árvore DOM flutuante não repousa fisicamente dentro do popover que o gerou. Ela flutua sobre a base da tela (`<body>`).
Quando o usuário produzia um evento de clique no interior do elemento `<Select>`, esse evento de mouse (`mousedown`) borbulhava _up_ no DOM puro do SO/Browser. Os listeners nativos gerados pela primitiva `createClickOutside` — anexada no Menu Pai — reconheciam a própria tela como alvo do clique (já que não estavam no array dos seus "filhos documentados"), ordenando o fechamento abrupto dos dois componentes concorrentes.

**A Solução**:
Forçar a parada da propagação em eventos raiz de toque e clique (`e.stopPropagation()`). Contudo, aplicar de modo sintético via prop JSX Solid (`onMouseDown`) não resolvia pois a engrenagem nativa da primitiva escutava antes do ciclo da Synthetic Layer da biblioteca.
Substituiu-se então a injeção da referência do componente por uma abstração nativa via:
```tsx
element.addEventListener('mousedown', e => e.stopPropagation());
element.addEventListener('touchstart', e => e.stopPropagation());
```
Garantindo controle pleno sobre a propagação do evento ainda no portal da janela.

### Type Safety das Ações de Filtro (`<any>`)
**O Problema:** Os componentes `Select` e `ToggleGroup` na ListView devolviam em seus handlers genéricos `val: any` e injetavam-no sem tipagem aos *reducers* de `filters.setSortBy()` e `filters.setLayout()`.

**A Solução:** Adotou-se tipagem estrita via Casting literal para suprimir e refatorar ESLint warnings, ex: `val as 'modified_at' | 'added_at' ...`, bloqueando que regressões de novas UI's mandassem strings não preparadas.

---

## 🔮 Possíveis Melhorias Futuras

1. **Atalhos no Texto das Tooltips:**
   - Adicionar ao `content` das novas `Tooltips` o atalho respectivo de teclado (`Cmd+[`, `Cmd+]`, `Cmd+F` etc.) se existir registro ativo dentro da store de shortcuts (reuso do módulo `formatShortcutForDisplay()` na barra de navegação principal).
2. **Componentização de Toolbars Secundários:**
   - O array de botões e comportamentos da _Search Bar_ está inflando na UI Primária; ponderar migrar sub-blocos e sub-popovers para seções exclusivas.
3. **Migrar outros Menus Suspensos:**
   - Padronizar toda a aplicação (`ContextMenus` e painel de `Workspace`) para adotarem as lógicas polidas providas pela stack do `PopoverRoot`, já que essa tem se mostrado mais portável do que o legado das engrenagens pesadas do `DropdownMenu` primário em fluxos densos.
