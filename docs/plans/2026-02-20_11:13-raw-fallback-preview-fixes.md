# Resolução de Extração de Previews em Arquivos RAW Complexos

**Data:** 2026-02-20
**Hora:** 11:13
**Autor:** Antigravity (Assistant)

## 1. Contexto e Objetivo

O objetivo inicial desta tarefa foi estabilizar e aprimorar o comportamento do sistema de visualizações de ficheiros RAW da aplicação Mundam. Enquanto a geração em background das *Thumbnails* (cópias pequenas otimizadas) já funcionava, a funcionalidade crucial que responde aos **Previews** de alta-fidelidade consumidos na interface web apresentava falhas críticas (ícone quebrado) para certos ficheiros exóticos de câmeras, especificamente: `.x3f`, `.raw`, `.mef`, `.kdc`, `.iiq`, `.fff`, e `.3fr`.

A nossa meta era garantir uniformidade: independentemente da complexidade, estrutura ou restrição do formato RAW, o backend Rust deveria conseguir retornar pro Web View um binário decodificável em tela cheia com alta velocidade e qualidade sem crashar o DOM.

## 2. Diagnóstico e O Problema Original

Após a criação de rotinas de testes rigorosas rodando diretamente com as ferramentas legadas em formatos difíceis do diretório `file-samples/Imagens/RAW/`, levantámos o Root Cause (Causa Raiz) de todos as miniaturas ocultas quebradas em Previews:

1. **A Maldição de Metadados Corrompidos (Falso-Positivo do LibRaw)**:
   A biblioteca intermediária `rsraw` (que engloba o `LibRaw` C++) tem por papel navegar o arquivo buscando o byte stream primário associado à "Thumbnail". O problema é que em arquivos fotográficos ultradensos como Phase One `IIQ` e Hasselblad `3FR/FFF`, a assinatura apontada pelo bloco interno de hardware às vezes levava a um conjunto de dados brutos **indescritíveis**. O Rust considerava a extração um sucesso (`Ok(Vec<u8>)`), o MimeType detectava à moda antiga ("isso parece ser JPEG") e era despejado para a web. O Chrome repudiava o stream de bytes bizarros e silenciava a renderização de `<img />`.
2. **Browsers Não Falam TIFF (Limitação Web Natural)**: 
   Alguns desses formatos exóticos extraídos (como no caso dos Leicas antigos, Kodak `KDC` e Sigma `X3F`) não geravam previews JPEG; eles escondiam internamente representações em **RAW TIFF sem compressão**. O sistema antigo até percebia isso, mas empurrava com `mimetype="image/tiff"`. O browser/electron nativamente é incapaz de renderizar `.tiff` em tags de imagem.
3. **O Escaneamento Brute-Force Culpado do Truncamento**: 
   A rotina antiga `brute_force_extract_jpeg_data` tentava escanear binários bit-a-bit procurando JPEGs (`FF D8 FF`), e encerrava a busca na primeira quebra visual de (`FF D9`). Câmeras Sigma armazenam a *"thumbnail"* diminuta antes do *"preview HD"* nas entranhas dos the `.x3f`. Isso gerava um pacote amputado decepado pela metade, resultando mais uma vez num parse inválido pelo Chrome/Electron, deixando a tela em branco.

## 3. A Solução Implementada

Diante do conhecimento profundo de como a memória e a arquitetura decodificavam os dados, introduzimos uma resiliência total na etapa de `extract_raw_preview` (localizada em `src-tauri/src/thumbnails/extractors/mod.rs`):

### 3.1 Barreira Leve e Dinâmica de Validação (`is_valid_image`)
Para nos defendermos de dados ilegíveis da LibRaw, construímos uma interceptação ultraleve baseada na library abstrata `image`. Antes de qualquer payload de RAW ser devolvido como Preview para o Frontend, ele passa por `image::ImageReader::new(cursor).into_dimensions()`. 
* Isso invoca a leitura restrita ao *HEADER* dos bytes (não consome CPU processando os píxeis!).
* Se for falho, truncado ou inválido, o bloco `Ok` é rejeitado, caindo perfeitamente pros algoritmos de Fallback.

### 3.2 Transmutação On-The-Fly de TIFF em PNG
Caso o motor interno (tanto LibRaw quanto binário) ateste que o preview extraído tem a Magic Signature de um `TIFF` (`49 49 2A 00` / `4D 4D 00 2A`), nós o rasterizamos imediatamente em memória (`load_from_memory -> image::ImageFormat::Png`) dentro do Rust. A requisição já desce para o WebView convertida num Blob `image/png` perfeitamente web-ready. O Chrome volta a sorrir e exibir previews gloriosas dessas fotos cruas.

### 3.3. Refatoração do Scanner de Varredura de JPEG (Brute Force)
Corrigimos o leito de corte no `raw.rs / brute_force_extract_jpeg_data`. Expandimos a matriz de escanemento para avaliar todas as opções de JPEG possíveis num segmento denso de MemMap (até 15MB) e selecionar apenas a de maior peso/tamanho total. Ele garante que encontramos, de fato, a melhor miniatura possível não-despedaçada caso todas as bibliotecas padrão fracassem.

**A Cascata Definitiva Virou:**
1. **Fidelidade (LibRaw)** → Verifica integridade → se sucesso e não-Tiff, retorna; se Tiff, converte pra PNG.
2. **Resiliência Estruturada (Binary JPEG Scanner)** → Verifica integridade global de exif markers → extrai, converte se necessário ou envia.
3. **Tanque de Guerra (Brute Force Memory Scanner)** → Caçador sujo de hexadecimais de JPEG. Garante que nunca haverá recusa.

## 4. Obstáculos e Dificuldades Adicionais
- Fazer o debug unitário dos falhanços foi bem frustrante até notar que o Rust dizia que a extração estava "tudo OK", enquanto o browser se debatia.
- A library abstrata de conversão falhava perante headers de `.DS_Store` nas pastas RAW do Mac.
- Mimetypes duvidosos devolvidos pelo Extractor precisaram de checagem explícita dos Magic Bytes do payload em contraparte ao mimetype implícito. 

## 5. Passos Futuros e Melhorias
1. **Remoção de Dead Code (Clean-Up)**: Temos um aviso visual do linter no Rust avisando sobre `extract_embedded_jpeg` e `MdpLayerInfo` que estão órfãos. É recomendável passar um `cargo fix` ou remoção manual nos módulos em breve.
2. **Tratamento Específico RAW Video (BRAW/R3D)**: Os pipelines reescritos e estáveis lidam muito bem com imagem RAW. Quando a meta mover para RAW-Video, talvez seja bom separar completamente o `PreviewStrategy::RawVideo`, de forma isolada do FFmpeg, para não cruzar problemas das câmeras Mirrorless/DSLR.
3. **Cache Inteligente de Previews (Disk-backed)**: A varredura pesada (Memory Map Brute Force/Conversão TIFF) penaliza brevemente o I/O se o navegador pedir os Previews consecutivamente sem pausa num preview gigante de 4MB+. Considerar adicionar um mecanismo no Rust que devolva um arquivo `.webp` rápido (Thumbnail/Temp file) por ID em vez de gerar binariamente na rede, se a performance do scroll da UI começar a acusar.
