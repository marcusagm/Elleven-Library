# Sprint 11.3: Correção de Arquivos de Projeto (Arquitetura V2)

**Status da sprint:** Planejamento
**Data e hora de início da sprint:** -
**Data e hora da conclusão estimada:** -

## Objetivo

Restaurar a paridade total e eliminar erros de visualização em arquivos de projeto (Design e MindMaps) na Arquitetura V2.
1. Habilitar Previews e Metadados para arquivos Affinity (`.afdesign`, `.afphoto`, `.afpub`).
2. Corrigir falhas de visualização e metadados em arquivos Aseprite.
3. Resolver o erro de extração de metadados em arquivos XMind.
4. Corrigir a geração de thumbnails em arquivos Adobe Illustrator (.ai) que possuem apenas stream PDF.
5. Melhorar a qualidade e fidelidade das thumbnails para formatos baseados em ZIP (Figma, Krita).

## Análise de Gap e Problemas Identificados

Com base na verificação manual (`task-list-end-new-backend.md`):

### Affinity & Adobe Illustrator
| Extensão | Problema Identificado | Impacto | Causa Provável |
| :--- | :--- | :--- | :--- |
| `afdesign/photo/pub` | Erro de metadados; Preview não funcional. | 🟠 Grave | `AffinityFormatProvider` não implementa `MetadataCapability` nem `PreviewCapability`. |
| `ai` | Não gera thumbnails (apenas preview funciona). | 🟠 Grave | O extrator retorna PDF, mas o sistema de thumbnails não renderiza o PDF para imagem. |

### Aseprite & XMind
| Extensão | Problema Identificado | Impacto | Causa Provável |
| :--- | :--- | :--- | :--- |
| `ase`, `aseprite` | Maioria das thumbnails ausente; sem preview. | 🟠 Grave | `AsepriteFormatProvider` não implementa `PreviewCapability`. |
| `xmind` | Thumbnail correta, mas erro de metadados no preview. | 🟠 Grave | `XMindFormatProvider` não implementa `MetadataCapability`. |

### Outros Projetos
| Extensão | Problema Identificado | Impacto | Causa Provável |
| :--- | :--- | :--- | :--- |
| `fig` | Thumbnail e preview de baixíssima qualidade. | 🟡 Médio | Extração da thumbnail de baixa resolução do ZIP sem upscale Lanczos3. |
| `sai` | Dimensões extraídas parecem incorretas (pegando da thumb). | 🟡 Médio | Falha no parsing do cabeçalho binário original do SAI1. |

---

## Referências Técnicas

### V1 (Legacy)
- `src-tauri/src/thumbnails/affinity.rs`: Scanner binário para PNGs gigantes no final do arquivo.
- `src-tauri/src/thumbnails/extractors/ai.rs`: Estratégia PDF -> XMP -> Binary.
- `src-tauri/src/thumbnails/extractors/aseprite.rs`: Suporte a GIF animado para múltiplas frames.

### V2 (Current)
- `affinity_format.rs`: Stub básico (metadados marcados como "skipping for now").
- `aseprite_format.rs`: Possui extração técnica básica, mas falta preview.
- `xmind_format.rs`: Implementa apenas thumbnail.
- `project_zip_formats.rs`: Handler genérico para Krita, Figma, etc.

---

## Tarefas do Plano de Ação

### 1. Suporte Affinity Completo (`AffinityFormatProvider`)
- [ ] **Metadata Extraction**: Implementar `MetadataCapability` para retornar pelo menos o nome do formato e dimensões (se possível via parsing do header).
- [ ] **Preview Capability**: Implementar `PreviewCapability` reutilizando o `extract_largest_png`. Isso eliminará o erro de "metadata extraction" ao abrir o preview.
- [ ] **Optimization**: Garantir que o scanner binário use `memmap2` para arquivos Affinity muito grandes (>500MB).

### 2. Restauração Aseprite (`AsepriteFormatProvider`)
- [ ] **Animated Preview**: Implementar `PreviewCapability` que retorna o GIF animado gerado pelo extrator.
- [ ] **Thumbnail Debug**: Investigar por que algumas thumbnails falham (verificar limites de memória no `tokio::task::spawn_blocking`).
- [ ] **Metadados**: Adicionar contagem de frames e camadas aos metadados técnicos.

### 3. Ajustes em XMind e AI
- [ ] **XMind Metadata**: Implementar `MetadataCapability` simples para `XMindFormatProvider` para sanar o erro de interface.
- [ ] **AI Thumbnails**: Assim como na Sprint 11.2, integrar a renderização de PDF para imagem no `AiFormatProvider::generate` quando o stream detectado for PDF.
- [ ] **AI Metadados**: Tentar extrair a versão do Illustrator do header do arquivo.

### 4. Melhoria de Qualidade em Projetos ZIP
- [ ] **Figma/Krita**: Aplicar `Lanczos3` upscale se a thumbnail extraída do ZIP for menor que o `size_hint` solicitado.
- [ ] **Deduplicação de Metadados**: Garantir que dimensões extraídas de arquivos `.clip` e `.sai` venham do banco de dados interno do arquivo, não da imagem de thumbnail.

---

## Critérios de Aceitação

- [ ] Arquivos Affinity abrem o preview sem erros de metadados.
- [ ] Arquivos Aseprite mostram previews animados (GIF) quando possuem múltiplos frames.
- [ ] Arquivos XMind não apresentam erro de "metadata extraction" ao abrir.
- [ ] Arquivos AI exibem thumbnails no Grid (mesmo os baseados puramente em PDF).
- [ ] Dimensões de arquivos `.sai` e `.clip` refletem o tamanho real do canvas.

---

## Riscos e Mitigações

| Risco | Impacto | Mitigação |
| :--- | :--- | :--- |
| **GIF Performance** | GIFs animados gigantes de Aseprite podem consumir muita RAM. | Limitar o número de frames processados para a thumbnail ou reduzir a resolução do GIF de preview. |
| **Affinity Binary Scan** | Escanear arquivos de vários GB pode ser lento. | Priorizar a busca no final do arquivo (onde a Affinity costuma salvar previews) e usar buffers otimizados. |
| **Fidelidade de Cores AI** | PDF streams podem ter perfis de cores (CMYK) que o sistema não converte bem. | Utilizar o `zune-jpeg` com swap de canais se detectado perfil Adobe. |
