# Análise Técnica: Suporte ao Formato Rebelle (.reb) no Mundam

Este relatório descreve a estrutura do formato de arquivo do software Rebelle e define a estratégia para extração de thumbnails e visualizações de alta qualidade.

---

## 1. Estrutura do Formato

O arquivo `.reb` é um **pacote ZIP** (identificado pelo magic number `PK\x03\x04`). Ele funciona como um contêiner para diversos ativos que compõem a pintura digital.

### Conteúdo Típico do Pacote:
| Arquivo | Descrição |
| :--- | :--- |
| `artwork.xml` | Metadados da pintura (dimensões, camadas, histórico de cores). |
| `canvas.png` | **Imagem composta final** (renderização completa da arte). |
| `layerN.png` | Imagem individual de cada camada (onde N é o índice). |
| `layerN_flow.dat` | Dados proprietários de simulação de fluidos/fluxo. |
| `paper.png` | Textura do papel/tela utilizada. |
| `profile.icc` | Perfil de cores ICC para gerenciamento de cores. |

---

## 2. Estratégia de Extração

Diferente de outros formatos que embutem thumbnails proprietárias em chunks binários, o Rebelle utiliza arquivos de imagem padrão dentro do ZIP, o que facilita a implementação.

### Geração de Thumbnail
*   **Fonte:** O arquivo `canvas.png` dentro do ZIP.
*   **Processo:** 
    1.  Abrir o arquivo `.reb` como um arquivo ZIP.
    2.  Localizar e extrair o fluxo de dados de `canvas.png`.
    3.  Redimensionar para o tamanho padrão de thumbnail do Mundam (ex: 256x256).
*   **Vantagem:** O `canvas.png` já representa a arte final com todas as camadas e efeitos aplicados.

### Visualização de Alta Qualidade (Preview)
*   **Fonte:** O próprio `canvas.png`.
*   **Processo:** Como o `canvas.png` é a renderização em resolução total da obra, ele deve ser usado diretamente para a visualização de alta qualidade.
*   **Otimização:** Para arquivos extremamente grandes, pode-se gerar uma versão intermediária em cache, mas a fonte primária é sempre o `canvas.png`.

---

## 3. Plano de Implementação (Padrões Mundam)

### Backend (Rust/Tauri)
1.  **Novo Extrator:** Criar `src-tauri/src/thumbnails/extractors/rebelle.rs`.
2.  **Dependências:** Utilizar a crate `zip` para leitura do contêiner e `image` para processamento (se necessário upscale/downscale).
3.  **Lógica:**
    ```rust
    pub fn extract_rebelle_preview(path: &Path) -> Result<Vec<u8>, RebelleError> {
        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut canvas_file = archive.by_name("canvas.png")?;
        let mut buffer = Vec::new();
        canvas_file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    ```

### Frontend (SolidJS)
1.  **Registro de Formato:** Adicionar `.reb` à lista de formatos suportados em `definitions.rs`.
2.  **Componente de Visualização:** O `ImageInspector` já deve ser capaz de lidar com o buffer retornado, pois é um PNG padrão.

---

## 4. Considerações de Performance
*   Arquivos `.reb` podem ser grandes (centenas de MB). A extração do `canvas.png` deve ser feita via streaming ou leitura parcial do ZIP se possível, para evitar carregar todo o arquivo em memória.
*   O cache de thumbnails é essencial para evitar a descompressão repetida do ZIP durante a navegação no grid.
