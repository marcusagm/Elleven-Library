# Sprint 10.5: Migração Completa — Extractor CorelDRAW (.cdr)

**Status da sprint:** Verificação necessária
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Garantir paridade completa do extractor CorelDRAW (`.cdr`) na V2, incluindo thumbnail embarcado, metadados técnicos e preview em alta resolução.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/coreldraw.rs`
- **Tamanho:** 18,425 bytes
- Parse completo do formato RIFF de arquivos CDR
- Extração do bloco `RIFF → DISP` ou `RIFF → bmDt` para thumbnail
- Suporte a CDR v16+ (mini-bitmap embutido) e versões antigas
- Extração de metadados via bloco `INFO`

### V2 — `mundam-main/src-tauri/src/processing/media/extractors/coreldraw.rs`
- **Tamanho:** 7,459 bytes (−60% vs V1)
- Implementação parcial — sinaliza gap significativo

## Análise de Gap

| Funcionalidade | V1 | V2 |
|---|---|---|
| Parse RIFF container | ✅ | ❓ Verificar |
| Extract bloco `DISP` (thumbnail) | ✅ | ❓ Verificar |
| Extract bloco `bmDt` (bitmap data) | ✅ | ❓ Verificar |
| Suporte multi-versão CDR | ✅ | ❓ Verificar |
| Extract `INFO` (título, autor) | ✅ | ❓ Verificar |
| Dimensões do documento | ✅ | ❓ Verificar |

## Tarefas

### 1. Auditar coreldraw.rs V2 vs V1

**Status:** Pendente

Comparar linha a linha os dois extractors. Identificar quais casos não estão cobertos no V2.

**Arquivos a comparar:**
- `mundam-main/src-tauri/src/thumbnails/extractors/coreldraw.rs` (V1, 18KB)
- `src-tauri/src/processing/media/extractors/coreldraw.rs` (V2, 7.5KB)

### 2. Portar Parse RIFF Completo

**Status:** Pendente (se necessário após auditoria)

O formato CDR é um container RIFF. O V1 tinha um parser RIFF completo com chunked traversal. Porta a lógica que o V2 estiver perdendo.

**Estrutura do CDR:**
```
RIFF 'CDR '
  ├── DISP (Windows Device Independent Bitmap — thumbnail)
  ├── LIST 'PDTA'
  │   └── bmDt (bitmap data comprimido)
  ├── LIST 'INFO'
  │   ├── INAM (nome do arquivo)
  │   └── IART (criador)
  └── vrsn (versão do CDR)
```

### 3. Adicionar Suporte a .cdr no BinaryDesignFormatProvider ou Provider Separado

**Status:** Verificar

O arquivo `.cdr` (CorelDRAW) não está na lista de `supported_extensions` do `BinaryDesignFormatProvider` atual:
```rust
fn supported_extensions(&self) -> Vec<&'static str> {
    vec!["sai", "sai2", "xcf", "rif", "riff", "clip"] // sem "cdr"
}
```

**Verificar se existe um `CorelDrawFormatProvider` separado** ou se precisa adicionar `.cdr` ao `BinaryDesignFormatProvider`.

### 4. MetadataCapability e PreviewCapability para CDR

**Status:** Pendente

Implementar extração de dimensões e preview em alta resolução para CDR, seguindo o mesmo padrão da sprint 10.3.

## Arquivos a Modificar

- `src-tauri/src/processing/media/extractors/coreldraw.rs` — portar implementação completa
- `src-tauri/src/processing/media/binary_design_formats.rs` — adicionar suporte a `.cdr`

## Critérios de Aceitação

- [ ] Arquivo `.cdr` gera thumbnail correto
- [ ] Suporte a múltiplas versões do CDR (v7, v12, v16+)
- [ ] Inspector mostra dimensões (largura × altura do documento)
- [ ] Preview modal mostra thumbnail em alta resolução

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/coreldraw.rs`
