# Guia de Implementação: Format-Kit e Migração de Mídias Específicas

## 1. O Problema Atual vs O Novo Padrão

No arranjo atual do Mundam (em `formats/definitions.rs`), possuímos dezenas de instâncias de `FileFormat` amontoadas com *enums* de estratégias globais (Ex: `ThumbnailStrategy::NativeImage` ou `ThumbnailStrategy::Icon`). Embora útil de início, essa abordagem de "famílias" se esgota rápido: um arquivo `.PSD` e um `.ZIP` do CLIP STUDIO PAINT são ambos "Projetos", mas são dissecados de maneiras brutalmente opostas por trás das cortinas no C++. 

Sob o escudo do **Format-Kit Registry** (Arquitetura Hexagonal), cada formato peculiar ganhará sua própria Classe Ativa (um *Struct FormatProvider* próprio). O servidor não agrupa formatos como "Vídeos" cega ou genericamente. O formato atende a Trait de *ThumbnailCapability* do **seu jeito**, seja chamando imagem binária C++, ou chamando CLI nativa, garantindo que o acoplamento do *Photoshop* jamais interfira no acoplamento do formato de *LightWave Object*.

Este documento é o **SOP (Standard Operating Procedure)** para converter as antigas abstrações do `definitions.rs` em Plugins Puros do Format-Kit.

---

## Passo 1: Criação Singular do Módulo do Formato

Em vez de amontoarmos tudo, crie um arquivo isolado apenas para o algoritmo daquele formato (ou núcleo forte que o parseia). Exemplo prático: **Photoshop** (`.psd`).

1. Crie o arquivo: `src-tauri/src/processing/media/psd_format.rs`.
2. O Struct é a fachada (Adapter) para as bibliotecas de parsing.

```rust
use crate::core::formats::{FormatProvider, MetadataCapability, ThumbnailCapability};
use crate::core::error::AppResult;
use std::path::Path;

pub struct PhotoshopFormatProvider {
    // Pode conter a engine psd_rs subjacente pré-carregada / configurada
}

impl PhotoshopFormatProvider {
    pub fn new() -> Self {
        Self {}
    }
}
```

---

## Passo 2: Implementando a Identificação (FormatProvider - Otimizado)

O Struct assina a interface. Em nossa Arquitetura Hexagonal refinada, não iteramos *linearmente* para saber quem processa o quê. Cada provedor declara em alto-nível quais extensões suporta, permitindo que o Registry o coloque num Hash Map de acesso `O(1)` instantâneo.

```rust
#[async_trait]
impl FormatProvider for PhotoshopFormatProvider {
    fn name(&self) -> &'static str {
        "ADOBE_PHOTOSHOP_PSD"
    }

    /// Roteamento Veloz: O Cartório o encontrará instantaneamente quando o Indexador ler estas extensões
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["psd", "psb"]
    }

    /// Opcional: Para evitar processar um VÍDEO.psd falso
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"8BPS")
    }

    // Retorna 'self' porque este próprio struct implementará a Trait de Metadata
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self) 
    }

    // Retorna 'self' porque ele também sabe gerar Thumbnails
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}
```

---

## Passo 3: Implementando Metadados Feitos Sob Medida (Technical)

Diferente do vídeo que extrai taxa de quadros (FPS), a extração do PSD é puramente gráfica/textual. A interface normaliza a saída:

```rust
#[async_trait::async_trait]
impl MetadataCapability for PhotoshopFormatProvider {
    
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        // Usa `psd` ou bindings C++ isoladas aqui
        // let psd_file = psd::Psd::from_bytes(&std::fs::read(path)?)?;
        
        Ok(serde_json::json!({
            "color_mode": "CMYK", // ex: psd_file.color_mode()
            "layers_count": 14,
            "width": 3000,
            "height": 4500,
            "resolution_dpi": 300 
        }))
    }

    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        // Lê nomes de Layers e Textos aninhados para empoderar o Fuzzy Search.
        Ok(serde_json::json!({
            "layer_names": ["Background", "Character Glow", "Text: Bem Vindo"]
        }))
    }
}
```

---

## Passo 4: Implementando a Extração Visual (O Antigo 'NativeExtractor')

Onde antigamente o `definitions.rs` delegava via `strategy: ThumbnailStrategy::NativeExtractor` para um super-switch statement global, agora o próprio PsdFormatProvider traz a thumbnail pro ram.

```rust
#[async_trait::async_trait]
impl ThumbnailCapability for PhotoshopFormatProvider {
    
    async fn generate(&self, path: &Path, _size_hint: u32) -> AppResult<Vec<u8>> {
        // Exemplo: Ler Header embutido do PSD (Thumbnail Header Resource 1036)
        // Isso impede carregar um PSD de 4 GB na Mémoria apenas na RAM, lendo
        // seletivamente só os bytes do thumbnail.
        
        // let raw_thumb_bytes = psd_extractor::get_thumbnail(path)?;
        // transform() ...
        Ok(vec![...]) // Bytes Jpeg puros extraídos direto das vísceras do PSD
    }
}
```

---

## Passo 5: Injeção Múltipla Dinâmica (O Cartório Global)

Não há mais o constante estático gigantesco do começo do `definitions.rs`. A Fábrica Central engolirá plugins um por um. O Event Bus ou Handlers pedirão o suporte para a extensão "x".

```rust
use crate::processing::media::psd_format::PhotoshopFormatProvider;
use crate::processing::media::clipstudio_format::ClipStudioFormatProvider;
use crate::processing::media::ffmpeg_video_fallback::FfmpegVideoFormatProvider;

pub fn build_format_registry() -> FormatRegistry {
    let mut registry = FormatRegistry::new();
    
    // Adiciona plugins granulares Específicos PRIMEIRO..
    registry.register(Box::new(PhotoshopFormatProvider::new()));
    registry.register(Box::new(ClipStudioFormatProvider::new())); // O antigo ZIP Preview extractor
    
    // ..E adcionam Fallbacks Genéricos (Families) NO FINAL
    registry.register(Box::new(FfmpegVideoFormatProvider::new())); 
    
    registry
}
```

---

## Conclusões da Refatoração do Extrator

Com esta arquitetura rigorosa focada no Extrator Específico:
1. Formatos embutidos como o "Affinity Design", arquivados no `definitions.rs` antigo como *NativeExtractor*, não sujarão arquivos alheios. Se um dia a API nativa do macOS para ler AFDESIGN falhar, apenas a `affinity_format.rs` emitirá Erros via AppResult. O sistema SQLite jamais será trancado ou afetado, visto que as extrações sempre retornarão um JSON abstrato (CQRS) limpo.
