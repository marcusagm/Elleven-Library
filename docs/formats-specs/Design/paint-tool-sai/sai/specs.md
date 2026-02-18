# Technical Analysis: PaintTool SAI v1 (.sai)

## 1. Visão Geral do Formato

*   **Extension:** `.sai`
*   **Software:** SYSTEMAX PaintTool SAI (Version 1).
*   **Category:** Layered Raster Image / Encrypted Container.
*   **Magic Signature:** No plaintext magic. However, the first 4 bytes of all encrypted files are consistently `6C 22 3C 7A` (Little-Endian `0x7A3C226C`).
*   **Tamanho Típico:** 2 MB to 100+ MB.
*   **Variações:** Version 1.x files are encrypted; Version 2.x files (`.sai2`) use a different, unencrypted chunk-based structure.

## 2. Estrutura Binária Global

The file is divided into fixed-size **4096-byte pages**.

| Offset | Tamanho | Tipo | Nome do Campo | Descrição |
| :--- | :--- | :--- | :--- | :--- |
| `0x0000` | 4096 | `Table` | **Table Page 0** | Metadata and checksums for the first 511 data pages. |
| `0x1000` | 4096 | `Data` | **Data Page 1** | Usually allocated content. |
| `0x2000` | 4096 | `Data` | **Root FAT** | Start of the Virtual File System (VFS) directory. |
| ... | 4096 | `Data` | **Content** | Virtual files and folders. |
| `0x200000`| 4096 | `Table` | **Table Page 1** | Encrypted metadata for pages 512-1023 (occurs every 512 pages). |

## 3. Header Principal

The "header" is actually the first **Table Page**. It is encrypted and serves as the entry point to the Virtual File System.

*   **Structure (Decrypted):** A sequence of 512 entries (8 bytes each).
*   **Entry Layout:**
    *   `u32`: Page Checksum / Decryption Vector.
    *   `u32`: Next Page Index (for linked chains).
*   **Endianness:** Little-Endian.
*   **Table Span:** 512 pages. Every 512th page is a "Table" page dedicated to metadata for the subsequent 511 "Data" pages.

## 4. Estruturas Internas Identificadas

### 4.1. FAT Entry (File Allocation Table)
Located in the Root Directory (starting at Page 2) and subsequent directory pages.

| Offset | Tamanho | Tipo | Descrição |
| :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Flags** (Non-zero = Active entry). |
| `0x04` | 32 bytes| `ASCII`| **Name** (Null-terminated ASCII). |
| `0x26` | 1 byte | `u8` | **Entry Type** (`0x10` = Folder, `0x80` = File). |
| `0x28` | 4 bytes | `u32` | **Start Page Index**. |
| `0x2C` | 4 bytes | `u32` | **File Size** in bytes. |

### 4.2. Virtual Files
*   `canvas`: Contains global document metadata (resolution, layers).
*   `thumbnail`: Contains the pre-rendered preview.
*   `layers/`: A virtual folder containing individual layer data.

## 5. Endianness

*   **Little-Endian.**
*   **Evidence:** Validated by the successful decryption of page pointers and sizes using LE interpretation.

## 6. Compressão / Encriptação

### 6.1. Encryption
*   **Cipher:** XOR-based mask with rotation and chain additions.
*   **Key Table:** A static 256-word (1024 bytes) table embedded in the SAI executable.
*   **Algorithm (simplified):** 
    - Words are decrypted using `P[i] = C[i] XOR (V + KeySum(V))` or similar, where `V` is the previous ciphertext word or page index.
*   **Table vs Data:** Table pages use a different derivation for the initial vector (based on page index) compared to Data pages (based on the checksum stored in the corresponding table entry).

### 6.2. Compression
*   Virtual files (like `thumbnail`) are usually uncompressed raw data within the encrypted container.

## 7. Dados de Imagem (Thumbnail)

*   **Location:** Found at virtual path `/thumbnail`.
*   **Encapsulation:** Spans multiple pages if necessary, linked via the Table entries.
*   **Header (12 bytes):**
    *   `u32`: Width.
    *   `u32`: Height.
    *   `u32`: Magic `0x32334D42` (`BM32`).
*   **Pixel Format:** Raw **BGRA8** (Blue, Green, Red, Alpha).

## 8. Thumbnail / Preview Embutido

*   **Existe preview?** Yes, mandatory in standard saves.
*   **Como extrair:**
    1.  Decrypt Page 0 (Table).
    2.  Use the checksum from Table Entry 2 to decrypt Page 2 (Root FAT).
    3.  Scan FAT entries for the name `thumbnail`.
    4.  Follow the page chain (using Table entries) to collect all bytes of the `thumbnail` virtual file.
    5.  Skip the 12-byte header and parse as BGRA.

## 9. Metadados

*   **Virtual File system:** The structure itself is a metadata rich environment.
*   **Canvas metadata:** Found in the `canvas` virtual file.

## 10. Engenharia Reversa Estrutural

*   **VFS Container:** SAI v1 is not a simple image format but a custom filesystem. It prioritizes data integrity (via page checksums) and speed (via fixed-page random access).
*   **Linking:** Chains of pages are not stored in a central FAT like DOS; instead, each page's "next" pointer is stored in the Table page that oversees it.

## 11. Estratégia para Implementação de Parser

1.  **Initialize Key Table:** Load the 256-word SAI symmetric key.
2.  **Verify Alignment:** File must be a multiple of 4096.
3.  **Decrypt Page 0:** Necessary to find where everything else is.
4.  **Resolve VFS:** Start at Page 2, navigate directories recursively if needed.
5.  **Reassemble File:** Handle page chaining for files larger than 4096 bytes. Skip Table pages that interrupt the data stream every 512 blocks.

## 12. Pseudocódigo de Parser

```pseudo
function decrypt_sai(file):
    key_table = [...]
    page0 = decrypt_table(page[0], index=0)
    root_fat = decrypt_data(page[2], checksum=page0.entry[2].checksum)
    
    thumb_entry = find_in_fat(root_fat, "thumbnail")
    if not thumb_entry: return error
    
    raw_data = []
    current_idx = thumb_entry.page_index
    while current_idx != 0:
        table_page = get_table_for(current_idx)
        data = decrypt_data(page[current_idx], table_page.entry[current_idx % 512].checksum)
        raw_data.append(data)
        current_idx = table_page.entry[current_idx % 512].next_page
        
    header = parse_header(raw_data[0:12])
    pixels = raw_data[12:] # BGRA format
```

## 13. Estratégia para Geração de Thumbnail

*   **Abordagem:** Extraction from the VFS `/thumbnail` file.
*   **Complexidade:** High (due to decryption and VFS reassembly logic).
*   **Encoding:** Convertible to PNG/JPG by swapping Blue and Red channels to standard RGBA.

## 14. Estratégia para Visualização Básica

*   Since full image reconstruction requires decrypting and compositing thousands of tiles from the `layers/` folder, the embedded thumbnail is the primary source for visual representation.

## 15. Mapa Comparativo Entre Arquivos

| Arquivo | Tamanho | Root Page | Thumbnail Dim | Observações |
| :--- | :--- | :--- | :--- | :--- |
| `carinha rpg.sai` | 2.06 MB | 2 | 170x96 | Standard small file. |
| `Biga cartaz.sai` | 3.20 MB | 2 | 102x170 | Inverted aspect ratio. |
| `capirotu.sai` | 92.3 MB | 2 | 170x170 | Large document, spans many tables. |

## 16. Pontos Incertos

*   **Flags de FAT (Confiança: 70%):** Higher bits in flags likely store attributes like read-only or hidden, but only the zero/non-zero state for activity is consistently used by parsers.
*   **Page 1 (Confiança: 50%):** Often contains documentation or app-specific versioning headers but is not strictly part of the VFS tree.

## 17. Conclusão Técnica

The `.sai` format is a sophisticated, albeit closed, encrypted container. Its design suggests a focus on random-access efficiency for a low-memory 2000s era environment. Implementing a parser from scratch with no reference requires heavy reverse engineering of the XOR primitive; however, with the symmetric key known, the VFS reassembly is a standard filesystem logic task.
