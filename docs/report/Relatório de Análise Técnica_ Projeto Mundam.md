# Relatório de Análise Técnica: Projeto Mundam

Este relatório apresenta uma análise aprofundada da arquitetura, qualidade de código e problemas técnicos identificados no projeto Mundam, com foco especial no tratamento de formatos de imagem e sistemas de entrada.

---

## 1. Análise de Formatos de Imagem e Visualização

### 1.1. Formatos Raster (XCF, MDP, SAI, SAI2)
**Problema Identificado:** O sistema atual utiliza extratores nativos que buscam apenas a thumbnail embutida nos arquivos. Isso resulta em visualizações de baixa qualidade (geralmente 256px ou menos) mesmo quando o usuário solicita uma visualização ampliada.

| Formato | Estado Atual | Problema de Qualidade | Recomendação |
| :--- | :--- | :--- | :--- |
| **XCF (Gimp)** | Extrator manual de camadas e composição Porter-Duff. | Extrai apenas o que está no buffer de tiles, limitado à resolução da thumbnail. | Implementar suporte a `xcf-view` ou usar `gegl` via CLI se disponível para renderização total. |
| **MDP (FireAlpaca)** | Parser de blocos PAC e XML. | Foca na extração do bloco "thumbnail". | O formato MDP armazena camadas comprimidas em zlib. A arquitetura de composição já existe no código, mas precisa ser escalada para a resolução total. |
| **SAI (v1)** | Descriptografia de páginas e VFS. | Extrai apenas a entrada `/thumbnail`. | O SAI v1 não armazena uma imagem composta de alta qualidade; a renderização total exigiria um motor de pintura completo, o que é inviável. **Sugestão:** Manter thumbnail, mas avisar o usuário. |
| **SAI2** | Parser de chunks e JSSF. | Problemas na detecção de chunks e falta de suporte a DPCM. | O código atual falha ao encontrar chunks `thum` em versões mais recentes. É necessário atualizar os offsets de busca e implementar o decodificador DPCM para thumbnails lossless. |

### 1.2. Formatos Vetoriais (AI, EPS)
**Problema Identificado:** A dependência de "scanners binários" para encontrar JPEGs embutidos falha em arquivos salvos sem compatibilidade com PDF ou sem preview embutido.

*   **AI (Illustrator):** O extrator `extract_ai_pdf` é robusto para arquivos modernos, mas falha em arquivos legados.
*   **EPS:** A tentativa de usar FFmpeg/Ghostscript é instável em ambientes sem as dependências de sistema corretas.
*   **Melhoria:** Integrar uma biblioteca de renderização vetorial leve ou garantir que o fallback para `binary_jpeg` seja a última opção, priorizando a renderização via `pdfium` para o stream PDF interno do AI.

---

## 2. Navegação e Atalhos de Teclado

### 2.1. Problemas de Arquitetura de Input
O sistema de input utiliza um modelo de "Escopos" (Scopes), mas apresenta falhas de vazamento de eventos:

1.  **Conflito de Foco:** Atalhos globais (como `Cmd+K` para busca) continuam ativos mesmo quando o usuário está digitando em campos de texto, a menos que o componente bloqueie explicitamente.
2.  **Navegação no Grid:** O hook `useGridKeyboardNav` gerencia o foco virtual, mas o foco nativo do navegador (`document.activeElement`) nem sempre está sincronizado, quebrando a acessibilidade (A11y).
3.  **Code Smell:** O componente `Input.tsx` possui lógica manual de `stopPropagation` para uma lista fixa de teclas (`Enter`, `Arrows`, etc.). Isso deveria ser gerenciado pelo sistema de escopos de forma declarativa.

### 2.2. Melhorias Propostas
*   **Escopo de Edição Automático:** Refatorar o `Input` para que, ao receber foco, ele ative um escopo de prioridade máxima que bloqueie todos os atalhos de caractere único.
*   **Navegação Espacial:** O grid atual usa uma lógica linear. Implementar navegação espacial real (considerando coordenadas X/Y) para lidar com layouts de alvenaria (masonry) de forma mais intuitiva.

---

## 3. Arquitetura e Code Smells

### 3.1. Backend (Rust)
*   **Tratamento de Erros:** Muitos extratores usam `Box<dyn std::error::Error>`. Seguindo as diretrizes do projeto, estes devem ser convertidos para enums `thiserror` específicos para permitir melhor tratamento no frontend.
*   **Documentação:** Faltam seções de `# Errors` em funções públicas de extração, violando o `backend-rust.md`.
*   **Performance:** A decodificação de imagens XCF/MDP é feita inteiramente na thread principal do Tauri. Mover para `tokio::task::spawn_blocking`.

### 3.2. Frontend (SolidJS)
*   **Reatividade:** Identificado uso de desestruturação de props em alguns subcomponentes, o que quebra a reatividade do Solid. Utilizar sempre `splitProps`.
*   **Visualização:** O `ImageInspector` carrega a thumbnail mesmo para visualizações grandes. Implementar um sistema de "Dual Loading": carrega a thumbnail instantaneamente e faz o upscale/renderização da imagem completa em background.

---

## 4. Plano de Ação Recomendado

1.  **Curto Prazo:** Corrigir os offsets do parser SAI2 e implementar o bloqueio de escopo global no componente `Input`.
2.  **Médio Prazo:** Implementar renderização de alta qualidade para MDP (usando as camadas já mapeadas) e XCF.
3.  **Longo Prazo:** Migrar todos os comandos Tauri para a arquitetura de "comandos finos" com serviços especializados em Rust.
