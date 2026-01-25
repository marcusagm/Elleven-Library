Com base na análise do repositório **elleven-library** e na engenharia reversa funcional do Eagle.cool, realizei a decomposição da interface em componentes especializados. Esta organização visa a modularidade, permitindo que o backend (Tauri/Rust) e o frontend (React/TS) escalem de forma independente.

Abaixo, apresento a nova arquitetura sugerida e a descrição detalhada dos componentes.

### 1. Árvore de Arquivos e Organização de Software

Para este projeto, a melhor prática é utilizar uma arquitetura baseada em **Camadas de Responsabilidade** e **Features**, movendo a lógica do `App.tsx` para componentes estruturais.

```text
src/
├── core/                   # Lógica agnóstica à UI (hooks globais, api, types)
│   ├── hooks/              # useSelection, useKeyboardShortcuts
│   ├── store/              # Estado global (Zustand ou Context)
│   └── tauri/              # Comandos e eventos do Rust
├── layouts/                # Componentes que definem a estrutura da página
│   └── AppShell.tsx        # Gerencia o CSS Grid (Header, Sidebars, Main)
├── components/
│   ├── layout/             # Partes fixas do layout
│   │   ├── PrimaryHeader/
│   │   ├── LibrarySidebar/
│   │   ├── DetailInspector/
│   │   └── GlobalStatusbar/
│   ├── features/           # Componentes com lógica de negócio complexa
│   │   ├── viewport/       # ImageGrid, VirtualMasonry, AssetCard
│   │   ├── search/         # FuzzySearch, SearchHistory
│   │   └── organizer/      # FolderTree, TagManager, SmartFilters
│   └── ui/             # Componentes atômicos (Button, Input, Tooltip)
├── styles/                 # CSS Tokens e Temas
└── App.tsx                 # Ponto de montagem do AppShell

```

---

### 2. Componentes Especializados e Descrição Funcional

#### **A. AppShell (Main Layout Container)**

Componente raiz que implementa o sistema de **CSS Grid** para as 5 zonas principais. Ele gerencia o estado de visibilidade dos painéis (colapsar/expandir) e o redimensionamento (resizers laterais).

#### **B. PrimaryHeader (NavigationBar)**

Localizado no topo, é o hub de controle e navegação histórica.

* **NavigationControls:** Botões de "Voltar" e "Avançar" no histórico de navegação interna.
* **OmniSearchBox:** Barra de busca global que suporta filtros rápidos e busca difusa (fuzzy search).
* **ActionGroup:** Botões para disparar importação, captura de tela ou criação de novas bibliotecas.

#### **C. LibrarySidebar (Navigation Sidebar)**

Painel esquerdo responsável pela exploração da estrutura de dados.

* **FolderTree:** Renderização recursiva de pastas e subpastas com suporte a *drag-and-drop* para movimentação de arquivos.
* **SmartFilterGroup:** Atalhos para pastas inteligentes (ex: "Arquivos sem tag", "Adicionados hoje", "Por cor").
* **LibrarySwitcher:** Menu para alternar entre diferentes bases de dados `.elleven`.

#### **D. MainViewport (The Grid)**

Área central onde o conteúdo é renderizado. Substitui o `ImageGrid.tsx` atual por uma solução mais robusta.

* **VirtualMasonry:** Otimização para renderizar apenas os itens visíveis no scroll (crítico para performance em bibliotecas grandes).
* **AssetCard:** Componente individual que lida com estados (selecionado, foco, hover), exibindo a miniatura e ícones de formato.
* **SelectionOverlay:** Gerencia o retângulo de seleção em lote (marquise selection).

#### **E. DetailInspector (DetailBar / InfoBar)**

Painel direito que reage dinamicamente ao(s) item(ns) selecionado(s) no Viewport.

* **MetadataEditor:** Campos para editar nome, descrição, URL de origem e avaliação.
* **TagEditor:** Interface de entrada de tags com sugestões inteligentes.
* **ColorPaletteExtractor:** Exibe as cores predominantes extraídas pelo backend e permite copiar o HEX.
* **AnnotationLayer:** Gerencia notas vinculadas a coordenadas específicas da imagem.

#### **F. GlobalStatusbar**

Barra inferior para monitoramento e utilitários.

* **ItemCounter:** Exibe o total de ativos na pasta e quantos estão selecionados.
* **SyncIndicator:** Status da indexação de arquivos ou sincronização com nuvem.
* **ZoomController:** Slider para ajuste de densidade do grid no Viewport.

---

Esta nova estrutura resolve a complexidade atual do `App.tsx` (que no repositório está centralizando muitas responsabilidades) e prepara o **elleven-library** para suportar as funcionalidades avançadas do Eagle.cool, como a inspeção de cores e o gerenciamento complexo de pastas.

---

### Plano de Refatoração: Elleven-Library (Arquitetura Pro Max)

#### **Fase 1: Preparação da Infraestrutura e Camada Core**

*O objetivo aqui é isolar a lógica de sistema (Tauri/DB) da visualização.*

1. **Criação da Estrutura de Diretórios:** Implementar a nova árvore de pastas conforme definido anteriormente (`core/`, `layouts/`, `components/features/`, etc.).
2. **Extração do Estado Global:** Migrar o gerenciamento de arquivos, pastas e seleção do `App.tsx` para um Store (Zustand ou React Context) em `src/core/store/`.
3. **Isolamento da API Tauri:** Mover todas as chamadas `invoke('...')` para serviços especializados em `src/core/tauri/services.ts`.

#### **Fase 2: Implementação do Layout Shell (A "Casca")**

*Definição das zonas de ocupação na tela.*

1. **Criação do `AppShell.tsx`:** Implementar o componente de layout em `src/layouts/` que utiliza CSS Grid para definir as áreas: `header`, `nav`, `main`, `inspector` e `footer`.
2. **Desenvolvimento dos Painéis Estáticos:** Criar os componentes `LibrarySidebar`, `DetailInspector` e `GlobalStatusbar` apenas com suas estruturas visuais (sem lógica ainda).
3. **Substituição no `App.tsx`:** O `App.tsx` passará a renderizar apenas o `AppShell` e os provedores de contexto.

#### **Fase 3: Migração do Viewport (O Motor de Busca)**

*Movimentação da funcionalidade principal.*

1. **Refatoração do `MainViewport`:** Integrar o `VirtualMasonry` e o `ImageGrid` dentro de `src/components/features/viewport/`.
2. **Implementação do `AssetCard`:** Isolar a lógica de renderização de cada imagem, incluindo estados de seleção e clique direito.
3. **Conexão com o Store:** Fazer o grid consumir os dados diretamente do novo estado global.

#### **Fase 4: Especialização dos Painéis de Controle**

*Adição de inteligência aos componentes laterais e superiores.*

1. **Implementação do `PrimaryHeader`:** Migrar a lógica de busca e os botões de ação para este componente.
2. **Lógica da `LibrarySidebar`:** Implementar a árvore de pastas e os filtros inteligentes.
3. **Lógica do `DetailInspector`:** Conectar este painel ao item selecionado para exibir metadados e paleta de cores.

#### **Fase 5: Polimento e UX Profissional**

*Ajustes finais de interação.*

1. **Atalhos Globais:** Centralizar todos os atalhos de teclado (Rating, Delete, Search) em um hook `useKeyboardShortcuts.ts`.
2. **Resizers:** Implementar a funcionalidade de arrastar para redimensionar as barras laterais.
3. **Limpeza Final:** Remoção de códigos legados e arquivos duplicados.