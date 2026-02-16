# Implementação: Suporte a Arquivos Rebelle (.reb)

**Data:** 2026-02-16
**Status:** ✅ Concluído
**Tipo:** Feature / Refatoração
**Módulos Afetados:** Backend (Rust)

---

## 1. Contexto

O objetivo desta tarefa foi adicionar suporte nativo a arquivos do software Rebelle (`.reb`) no Mundam. A análise preliminar revelou que arquivos `.reb` são containers ZIP que incluem uma imagem composta chamada `canvas.png`, ideal para geração de thumbnails e visualização.

## 2. Decisões Técnicas

Optou-se pela **Estratégia A (Extração Padrão em Memória)**:
- Utilizar a crate `zip` para abrir o arquivo.
- Extrair o arquivo `canvas.png` diretamente para um buffer de memória (`Vec<u8>`).
- Retornar este buffer como `image/png` para o pipeline de processamento de imagens do Mundam.

Esta abordagem foi escolhida pela simplicidade, robustez e baixo risco, dado que o `canvas.png` já está no formato correto e possui tamanho gerenciável.

## 3. Passos da Implementação

### 3.1. Criação do Extrator Especializado
Criado o arquivo `src-tauri/src/thumbnails/extractors/rebelle.rs` com a lógica de extração:

```rust
use std::path::Path;
use std::io::Read;
use std::fs::File;

pub fn extract_rebelle_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Tenta encontrar 'canvas.png' na raiz
    if let Ok(mut file) = archive.by_name("canvas.png") {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        return Ok((buffer, "image/png".to_string()));
    }

    // Fallback: busca case-insensitive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().eq_ignore_ascii_case("canvas.png") {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            return Ok((buffer, "image/png".to_string()));
        }
    }
    Err("No canvas.png found in Rebelle file".into())
}
```

### 3.2. Integração no Registro de Extratores
Atualizado `src-tauri/src/thumbnails/extractors/mod.rs` para incluir o novo módulo e a rota de extração:

```rust
// ...
pub mod rebelle;
// ...
match ext.as_str() {
    // ...
    "reb" => {
        rebelle::extract_rebelle_preview(path)
    },
    // ...
}
```

### 3.3. Definição do Formato
Atualizado `src-tauri/src/formats/definitions.rs` para registrar a extensão `.reb`:

```rust
FileFormat {
    name: "Rebelle Project",
    extensions: &["reb"],
    mime_types: &["application/x-rebelle"],
    type_category: MediaType::Project,
    strategy: ThumbnailStrategy::NativeExtractor,
    preview_strategy: PreviewStrategy::NativeExtractor,
    playback: PlaybackStrategy::None,
},
```

## 4. Validação e Teste

- **Compilação:** Executado `cargo check` para garantir integridade do código.
- **Correção de Borrow Checker:** Ajustada a lógica de `if let` no extrator para evitar erros de `mutable borrow` da crate `zip`.
- **Documentação:** Atualizado `README.md` (contagem de formatos) e o relatório técnico em `docs/report`.

## 5. Próximos Passos (Futuro)

- Monitorar performance em arquivos `.reb` muito grandes (>500MB).
- Considerar extração de metadados (`artwork.xml`) se houver demanda por filtragem baseada em tipo de papel ou versão do software.
