# Relatório de Análise: Gap de Features e Roadmap (Mundam)
> **Baseado em**: `docs/idea/features.md` vs. Implementação Atual
> **Data**: 10 de Fevereiro de 2026

Este documento detalha o que ainda precisa ser construído para alinhar o software atual com a visão original do Mundam, priorizando entrega de valor e estabilidade técnica.

---

## 1. Features Faltantes (Gap Analysis)

### 1.1 Análise Cromática e Busca por Cor
*   **Status**: ❌ Não Iniciado
*   **Visão**: O usuário deve conseguir buscar imagens por "Vibrante", "Escura" ou selecionar uma cor (ex: `#FF0000`) e ver imagens com essa tonalidade dominante.
*   **Requer**:
    *   ✅ *Realizado* - **Backend**: Extração de paleta via FFmpeg ou ImageMagick durante a geração de thumbnails.
    *   **Banco de Dados**: Tabela `image_colors` ou colunas JSON/Vetorial.
    *   **Frontend**: UI de `ColorPicker` no filtro de busca.

### 1.2 Web Clipper (Extensão de Navegador)
*   **Status**: ❌ Não Iniciado
*   **Visão**: Botão no Chrome/Edge para "Salvar Imagem no Mundam" ou "Salvar Página Completa".
*   **Requer**:
    *   **Backend**: Um servidor local (já existente em `server.rs` ou novo endpoint HTTP em porta fixa) para receber o payload (URL, Imagem Base64, Tags).
    *   **Extensão**: Projeto separado (Manifest V3) que se comunica com `localhost:9876`.

### 1.3 Exportação Inteligente (Empacotamento)
*   **Status**: ⚠️ Parcial (Apenas cópia simples)
*   **Visão**: Criar pacotes `.eale` ou `.zip` contendo as imagens + metadados (tags, notas, rating) para compartilhar com outros usuários do Mundam ou backup.
*   **Requer**:
    *   ✅ *Realizado* - **Backend**: Lógica de geração de JSON de manifesto + ZIP das imagens originais.
    *   **UI**: Modal de Exportação.

### 1.4 Suporte a Plugins / Scripts
*   **Status**: ❌ Não Iniciado
*   **Visão**: Permitir que usuários criem scripts JS/Lua ou actions para renomear arquivos em lote ou integrar com softwares 3D (Blender Bridge).
*   **Risco**: Alta complexidade de segurança (Sandbox). Pode ser postergado para v2.0.

---

## 2. Dívidas Técnicas Críticas (Technical Debt)

### 2.1 Migrações de Banco de Dados
*   **Severidade**: 🔴 Alta
*   **Ação**: Substituir lógica manual em `database.rs` por `sqlx migrate`.
*   ✅ *Realizado* - **Por que?**: Sem isso, adicionar a feature de "Cor" (que precisa de tabela nova) vai quebrar instalações existentes ou exigir código de migração manual propenso a falhas.

### 2.2 Refatoração do `LibraryStore` (Frontend)
*   **Severidade**: 🟠 Média
*   **Ação**: Mover a lógica de filtragem de árvore folder-by-folder para o Rust (`get_images_by_folder_recursive`).
*   ✅ *Realizado* - **Por que?**: Para bibliotecas pequenas (1k itens) o JS aguenta. Para profissionais (50k+ itens), a UI vai travar (jank) ao trocar de pasta.

---

## 3. Roadmap Sugerido

### Fase 1: Fundação & Estabilidade (Q1 2026)
> Objetivo: Garantir que o app não quebre com updates e preparar terreno para novos recursos.
1.  [Backend] Implementar `sqlx migrate` e limpar `database.rs`.
2.  [Backend] Implementar testes unitários para o `Indexer` e `Watcher`.
3.  [Frontend] Refatorar `VideoPlayer` para componentes menores.

### Fase 2: Visual Experience (Q2 2026)
> Objetivo: Implementar as "Killer Features" visuais.
1.  ✅ *Realizado* - [Backend] Criar pipeline de extração de cores em `thumbnails/mod.rs`.
2.  [Database] Atualizar schema para armazenar cores.
3.  [Frontend] Implementar Filtro por Cor na barra lateral.
4.  [Frontend] Melhorar Viewer 3D (suporte a texturas e iluminação básica).

### Fase 3: Conectividade (Q3 2026)
> Objetivo: Trazer conteúdo da web para dentro do app.
1.  ✅ *Realizado* - [Extensão] Criar "Mundam Clipper" MVP (Salvar imagem com clique direito).
2.  ✅ *Realizado* - [Backend] Criar endpoint `/ingest` no servidor local para receber do Clipper.
3.  [Frontend] Notificações Toast ao receber injeção externa.

## 4. Conclusão

O Mundam está com o "Core" (Visualização, Navegação, Performance Local) em excelente estado (cerca de 80% completo). Os 20% restantes são justamente as features que o diferenciam de um "File Explorer" comum (Cores, Clipper, IA Local). Focar na estabilidade do banco de dados agora é o passo mais inteligente antes de adicionar complexidade.
