# Relatório de Diagnóstico de Código e Arquitetura MUNDAM

**Data:** 04 de Março de 2026
**Objetivo:** Identificar melhorias, code smells, passivos técnicos e infrações aos guias do projeto (`docs/guidelines`), garantindo que a base de código do Mundam evolua de forma impecável rumo ao nível "State-of-The-Art" em Sistemas de *Digital Asset Management* (DAM).

---

## 1. Resumo Executivo

Após uma profunda varredura nos diretórios `src` e `src-tauri/src` utilizando as regras estabelecidas nos guias oficiais da aplicação (como `frontend-solid.md`, `backend-rust.md`, `core-architecture.md` e `documentation.md`), constatou-se que o MUNDAM obteve um **avanço gigantesco em segurança de tipagem e isolamento** (remoção impecável de `any` e `console.log`). 

Entretanto, para alcançar a excelência absoluta, o projeto ainda abriga **"god files"** (arquivos monolíticos), violações de padrões nominais (abreviações proibidas), quebras semânticas e alguns redutos de crash-vectors (panic local) na camada profunda de extração de mídia.

Abaixo segue o desdobramento granular destas anomalias separadas por disciplina.

---

## 2. Arquitetura, Modulação e "God Files"

Segundo os guias do projeto, é mandatório evitar arquivos que superem 300 linhas, sendo dever do desenvolvedor promover o particionamento das lógicas em Domínios ou *Compound Components*. 

### 🔴 Violações Ativas: Arquivos Monolíticos
Os seguintes arquivos ultrapassaram a tolerância arquitetural e devem ser os próximos alvos do padrão de refatoração *Strangler*:

**Frontend:**
- `src/components/features/viewport/layouts/VirtualListView.tsx` (420 linhas)
- `src/core/viewport/layout.worker.ts` (391 linhas)
- `src/core/store/metadata/tagActions.ts` (383 linhas)
- `src/core/input/dispatcher.ts` (363 linhas)
- `src/core/input/normalizer.ts` (354 linhas)
- `src/components/features/settings/KeyboardShortcutsPanel.tsx` (354 linhas)
- `src/components/ui/Table/Table.tsx` (353 linhas)
- `src/core/store/library/libraryActions.ts` (322 linhas)
- `src/components/features/search/useAdvancedSearch.ts` (327 linhas)
- `src/components/ui/VideoPlayer/useVideoPlayer.ts` (316 linhas)

**Backend:**
- `formats/definitions.rs` (1212 linhas) - Justificável em partes, mas precisa de revisão do Registry genérico.
- `streaming/server.rs` (677 linhas)
- `db/search.rs` (628 linhas)
- `indexer/watcher.rs` (483 linhas)
- `media/ffmpeg.rs` (376 linhas)
- `thumbnails/worker.rs` (374 linhas)

### 🔴 Uso de "Separadores Visuais" (Code Smell)
O guia `documentation.md` dita a **regra crítica:** *"Never use visual separators (e.g., `// ====` or `// ----`) to divide files into sections"*. O uso indica que o arquivo absorveu papéis adjacentes excessivos.

Foi detectado uso massivo dos marcadores `// ====` em:
- `viewportStore.ts`, `viewport/types.ts` e `viewport/schemas.ts`
- `input/store/inputStore.ts`, `input/index.ts` e `input/context.tsx`
- `viewport/layout.worker.ts`

**Solução Exigida:** Isolar os blocos divididos por esses blocos de comentários em módulos próprios exportáveis.

---

## 3. Confiabilidade e Robustez de Runtime (Backend)

O avanço na troca de lógicas inseguras pelo uso contínuo de `AppResult` foi fenomenal, protegendo quase toda a aplicação de Travamentos (Panics). Porém, o parser de arquivos ainda mantém gatilhos arriscados em caso de arquivos malformados entregues pelo FileSystem da máquina local.

**🔴 Ocorrências de `unwrap()` não tratadas identificadas:**
A regra `No unwrap()` do `backend-rust.md` foi ignorada em escopos adjacentes nos seguintes locais (podendo causar pânico fatal em processamentos assíncronos de extração de Thumbnail massiva):
- `src-tauri/src/thumbnails/extractors/mdp.rs` (Extrações seguidas de mime Types, 4 utilizações de unwrap em descompressão Zlib)
- `src-tauri/src/thumbnails/extractors/mod.rs` (No tratador principal de diretório na linha 464 e 465)
- `src-tauri/src/thumbnails/extractors/corel_painter.rs`
- `src-tauri/src/transcoding/cache.rs` (`assert!(audio_path.extension().unwrap() == "m4a")`)

**Solução Exigida:** Transacionar os retornos que usam `.unwrap()` via `match` ou propagar a falha usando operador `?` no retorno para serem sinalizadas e descartadas pelo novo "Poison Pill" da Worker Task, preservando a vida útil do software perante corrupções.

---

## 4. Práticas de Código, Semântica e Legibilidade

O MUNDAM exige uma legibilidade impecável sem atalhos cognitivos, prevendo manutenção a longo prazo sem fricção.

### 🔴 Abreviações Proibidas e Sufixos Antissistema
O `frontend-solid.md` decreta o banimento perene de qualquer abreviação (exemplo clássico sendo `Props`).
Centenas de retornos de tipagem no diretório `components/**` insistem na formatação de interface `XProps` em seu isolamento por Domínio de Tipos:
- `AssetMetadataProps`, `AudioPlayerProps`, `TableProps`, `ContextMenuProps`, etc.

**Solução Exigida:** Executar Search and Replace Global refatorando estas nomenclaturas de tipagem para incluir o termo absoluto: `ComponentNameProperties` em todo o projeto, como orienta o TSDoc Template.

### 🔴 Variáveis de Uma Letra Única ("Single-Letter Vectors")
Há proibição estrita de criar escopos varíaveis de único charlete (Ex: `i`, `j`). Esta convenção existe para que todo iterador ou matemática carregue sua própria justificativa existencial.
- `ItemView.tsx` (`const i = item()`)
- `GlyphsTab.tsx` (`for (let i = 33; i <= 126; i++)`)
- `GeneralPanel.tsx` (`const i = Math.floor...`)
- `useAudioPlayer.ts` (`for (let i = 0...`)
- `layout.worker.ts` (`for (let i = 0; i < items.length...`)

**Solução Exigida:** Renomear matrizes em laços para `index`, `charIndex`, ou `componentIndex`.

---

## 5. Dívidas Técnicas Menores Exaltadas

A aplicação carrega consigo os seguintes limitadores registrados com marcadores formais **`TODO`** que pedem atenção final para fechar o roadmap do MVP:
1. `src/core/input/context.tsx` (Add PointerProvider and GestureProvider)
2. `src-tauri/src/thumbnails/model.rs` (3D Thumbnailer implementation request)
3. `src-tauri/src/thumbnails/extractors/sai2.rs` (Implement DPCM decoding - Phase 2)

---

## 6. Integridade Estrutural e Pastas

Durante a avaliação de integridade física dos diretórios da aplicação (árvore `src/` e `src-tauri/src`), notou-se o acúmulo de redutos obsoletos e algumas quebras no rigor da modelagem de arquitetura limpa (Clean Architecture).

### 🔴 Incoerências Arquiteturais (Camada de Interface vs Framework)
O guia `core-architecture.md` define que os componentes de Interface Genérica (`src/components/ui`) devem ser burros (Dumb Components) e mantidos totalmente agnósticos a estado global e comunicação IPC (Backend). Contudo:
- O componente `src/components/ui/AudioPlayer/useAudioPlayer.ts` quebra o isolamento importando e consumindo diretamente o `@tauri-apps/api/core` (`invoke`).
**Solução Exigida:** Passar a ação de IPC via callbacks estritos pelos *Features* que invocam a *UI*, isolando para sempre a camada visual do framework de desktop.

### 🟢 [Resolvido] Pastas Vazias e "Dead Zones"
Na base de código foram detectadas as seguintes pastas completamente vazias, as quais devem ser estirpadas pela raiz ou adequadamente inicializadas para não sobrecarregar a esteira de Git e indexadores:
- `src/components/ui/Modal/components/`
- `src/assets/fileicons/`
- `src-tauri/src/bin/`

### 🔴 Código Obsoleto (Unused Dead Code)
Foram encontrados blocos de lógica de estado não acoplados ao ciclo vital da UI, mas que persistem adicionando peso e risco de refatorações falhas.
Exemplos detectados flutuando sem uso documentado:
- `src/core/hooks/gridNavHelpers.ts`
- `src/core/hooks/useMetadataNotifications.ts`

### 🔴 Dicotomia no Padrão `Utils`
Possuímos duas raízes de utilitários competindo indiretamente por lógica:
- `src/utils` (Formatadores textuais e de cor estáticos)
- `src/core/utils` (Orquestradores arquiteturais como `eventBus.ts` e `LifecycleManager.ts`)
Essa sobreposição enfraquece a estrutura. É sugerida a realocação das funções formatadoras para um domínio `src/core/formatters` ou correlato, extinguindo a genérica `/utils` raiz.

---

## 7. Parecer Final e Recomendações

O repositório já se encontra em um estado formidável de engenharia limpa devido à **total exterminação do tipo** `any` e de chamadas `console.log()` perdidas em escopos em produção, denotando grande preocupação com o profissionalismo do produto.

Para fechar o hiato entre a *Maturidade Funcional* e o estado *State-of-The-Art*:
1. **Limpeza da Base:** Extirpar as pastas vazias (`/Modal/components`, `/fileicons`) e os arquivos "Dead Code" (`gridNavHelpers.ts`) que minam as varreduras diárias.
2. **Isolamento de UI Component:** Retirar o hard-lock com a API do Tauri consumido indevidamente dentro das estruturas primitivas da biblioteca da própria UI (Ex: `useAudioPlayer.ts`).
3. **Quebra dos Stores e Viewport:** Destroçar o `layout.worker.ts` (391 linhas) e as actions grandes do Store para assegurar performance nativa em WebWorkers menores e melhor distribuição no tempo do Frame rate (TBT).
4. **Caçada Completa por `props`:** Transformar definitivamente o codebase para possuir sintaxe sem vícios do ecossistema React clássico, consolidando propriedades longas baseadas em TSDocs.
5. **Mapeamento Defensivo:** Eliminar de vez e com tolerância "zero" as fendas contendo `.unwrap()` em leitores binários e extratores no ecossistema Rust, que são as portas mais prováveis de quebra perante massas desconhecidas de dados brutos e arquivos corrompidos gerados pelo ambiente do cliente.
