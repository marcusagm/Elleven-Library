# Relatório de Resolução: Evolução, Refatoração e Fullscreen do VideoPlayer

## Visão Geral e Objetivo
O objetivo inicial foi realizar a refatoração e amadurecimento técnico do `useVideoPlayer.ts`, encurtando o arquivo (redução da contagem de linhas), extraindo lógicas isoladas para outros hooks e enriquecendo a escalabilidade. O segundo grande objetivo foi resolver problemas de integração do botão 'Fullscreen' do player de vídeo quando rodando englobado como um aplicativo nativo desktop macOS através do encapsulamento WebKit com o framework backend Tauri.

## 1. Refatoração Arquitetural e Modularização

### Divisão de Responsabilidades
Para seguir as diretrizes rigorosas do projeto e respeitar limites de complexidade cognitiva/extensão de linhas, o gancho gigante de estados de controle global de vídeo (`useVideoPlayer.ts`) foi quebrado em pequenas bibliotecas utilitárias de propósito único:
- `useFullscreen.ts`: Gerencia exclusivamente os estados do monitor e da proporção cheia da janela.
- `usePlayerVolume.ts`: Trata as funções de Mute e Slider vinculando-se com a *Global Store* de Áudio. 
- `useHlsAttachment.ts`: Responsável por acoplar as engrenagens da lib `HlsPlayerManager` ao invés da tag trivial de browser.

### Otimizações no Arquivo Inicial:
Essa decomposição purificou drasticamente o `useVideoPlayer.ts`, reduzindo suas responsabilidades verticais e mantendo-o apenas como um ponto de entrada/saída (um orquestrador central para o provedor SolidJS Context `VideoPlayerContext.tsx`).
Ao lidar também com limitações do Sonar/Linter na frente UI, a massiva função Switch-Case das teclas de atalho (`handleKeyDown`) em `index.tsx` foi removida proatividade e refatorada de "Complexidade Cognitiva 11" para 1, utilizando dicionário em mapa (`O(1)` em Action-Map keys de Teclado `event.code`).

### Documentação Exaustiva (TSDoc)
Adequando aos mandatos estipulados em `docs/guidelines/documentation.md`, assinaturas JSDoc/TSDoc impecáveis acompanhando os formatadores exigidos (`/** ... */`) e definindo `@params` e `@returns` foram incorporadas em:
- Todo Componente na UI do Componente de Vídeo (index, VideoControls, VideoSeekbar).
- A API do Contexto Interno e Exportação dos Provedores. 
- Scripts essenciais (`utils.ts`).

## 2. Obstáculos e Resolução do Fullscreen (Tauri 2 macOS)

A funcionalidade "Fullscreen" falhava sem comportamentos detectados na interface, apenas exibindo botões de troca interativa de estado mas preso nas amarras do CSS e do Container modal. O principal causador foi as limitações de permissões do Tauri na manipulação direta das propriedades da Desktop Window do host pelo motor WebView do sistema operacional.  

### Passo a Passo dos Problemas:
1. **Permissões Nativas Bloqueadas:** O App de frontend disparava chamadas pro WebKit de Maximizar Tela nativa, porem uma recusa interna gerava a string: `window.set_fullscreen not allowed. Permissions associated with this command: core:window:allow-set-fullscreen`.
   > Solução: Capacidades expostas em `src-tauri/capabilities/default.json` abrindo `"core:window:allow-set-fullscreen"` e `"core:window:allow-is-fullscreen"`.
2. **Conflito de Hierarquia (Camadas CSS sobrepondo ItemView):** Ao tentar contornar de antemão através unicamente do estilo injetando fixed positions `100vw/100vh` pro Player fugir das barreiras da WebView, isto colidia e roubava z-index do modal parent do Visualizador Padrão (`ItemView.tsx`).
   > Solução: Limpeza robusta. Removemos todos os hacks do `.ui-video-player-fullscreen` no app. O app do host gerenciará seu escopo sozinho ao engolir o Monitor todo.
3. **Incompatibilidade Inflexível com o Safari macOS WebKit (Bug de Renderização):** Mesmo ao assumir a tela inteira controlando API do Host, o `.item-view-overlay` persistia usando `width: 100vw`. O WebView no Mac simplesmente amordaça proporções baseadas em Viewport de continuarem o auto-crescimento do preenchimento fixo sem o devido "reflow", cortando e engolindo visualmente os controles da HUD inferior do Media Player. 
   > Solução: Em `item-view.css`, `width/height` baseados em `vw`/`vh` foram descartados em favor de ancoramento estrito `inset: 0` + métrica percentuais puras de 100% que expandem baseada nos fluxos do wrapper nativo do Cocoa Window sem dependência da flexibilidade da viewport de aba. 

### Sincronizando o Lifecycle do Tauri
Foi mapeada em `useFullscreen.ts` uma "Flag sentinela" indicando `didForceFullscreen`. Ela permite que se o usuário clicar no "X" para **Fechar o modal de Visualização de Vídeo (ItemView)** ocorrendo o destrutivo `onCleanup` do `VideoPlayer`, o gerenciador irá automaticamente puxar as persianas solicitando explicitamente para que API nativa do host Desktop **saia do modo de tela cheia**, preservando a qualidade de vida. Adicionalmente, também não interferirá se o próprio utilizador apertar o botão nativo do App Maximize do macOS que não deve ser revertido pelo Vídeo.

## 3. Melhorias e Evolução de Código Interno (Lint via SolidJS Dev Tools)
Na varredura e compilação do TypeScript Lint para controle rígido no pipeline do Vite, o Solid apontava Warning graves (Reatividade Silenciosamente Falha) quando repassávamos Argumentos Diretos nos Clones do `VideoPlayerContext`:
- Resolvido refazendo o injetor das `Props` utilizando getter Properties estritas de escopos. (`get props() { return props; }`), permitindo Tracking das reações mutadas corretamente em escopo isolado. 

## Conclusões / Possíveis Melhorias Futuras
- **Testes Unitários da Web API de Vídeo:** Mocar as APIs do Tauri para teste do hook de fullScreen puro.
- **Suporte Híbrido para Web-Builds:** Quando a aplicação evoluir para builds web puro dentro de abas de navegadores tradicionais com o servidor Rust em segundo plano, injetar a detecção do `document.documentElement.requestFullscreen()` como Fallback Web primário a frente do try-catch originado da proteção anti-crash atual providenciada.
- **Micro-interações no Mobile/Touch-Display:** Estender eventos para possibilitar entrada/saída do Fullscreen ao interagir com Tap-Gestures no meio da interface gráfica do Desktop/Mobile em construções Touch futuras.
