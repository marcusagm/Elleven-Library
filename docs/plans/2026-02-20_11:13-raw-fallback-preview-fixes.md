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
2. **Resiliência Estruturada (Binary JPEG Scanner)** → Verifica integridade global de exif markers → extrai, converte se necessário ou envia. Agora com suporte ilimitado a contêineres TIFF se a assinatura for encontrada no início do arquivo (como ocorre em arquivos RAW muito massivos tipo Phase One IIQ ou formatos médios acima de 60 MB), evitando truncamento.
3. **Tanque de Guerra (Brute Force Memory Scanner)** → Caçador sujo de hexadecimais de JPEG. Garante que nunca haverá recusa.
4. **Resgate via Hardware (FFmpeg Thumbnail Fallback)** → Caso todas falhem para thumbnails, envoca o `generate_thumbnail_ffmpeg_full`, com timeout dilatado para 45 segundos, que suporta encodings e metadados densos sem estourar o limite.

## 4. Obstáculos e Dificuldades Adicionais
- Fazer o debug unitário dos falhanços foi bem frustrante até notar que o Rust dizia que a extração estava "tudo OK", enquanto o browser se debatia.
- A library abstrata de conversão falhava perante headers de `.DS_Store` nas pastas RAW do Mac.
- Mimetypes duvidosos devolvidos pelo Extractor precisaram de checagem explícita dos Magic Bytes do payload em contraparte ao mimetype implícito. 

## 5. Passos Futuros e Melhorias
1. ~**Remoção de Dead Code (Clean-Up)**: Temos um aviso visual do linter no Rust avisando sobre `extract_embedded_jpeg` e `MdpLayerInfo` que estão órfãos. É recomendável passar um `cargo fix` ou remoção manual nos módulos em breve.~ *(Resolvido: Limpamos o dead code, excluímos vars unused no extractor do MDP, apagamos as listas do obsoleto Minolta MDC e eliminamos todos os warnings do compilador).*
2. **Tratamento Específico RAW Video (BRAW/R3D)**: Os pipelines reescritos e estáveis lidam muito bem com imagem RAW. Quando a meta mover para RAW-Video, talvez seja bom separar completamente o `PreviewStrategy::RawVideo`, de forma isolada do FFmpeg, para não cruzar problemas das câmeras Mirrorless/DSLR.
3. **Cache Inteligente de Previews (Disk-backed)**: A varredura pesada (Memory Map Brute Force/Conversão TIFF) penaliza brevemente o I/O se o navegador pedir os Previews consecutivamente sem pausa num preview gigante de 4MB+. Considerar adicionar um mecanismo no Rust que devolva um arquivo `.webp` rápido (Thumbnail/Temp file) por ID em vez de gerar binariamente na rede, se a performance do scroll da UI começar a acusar.


## 6. Benchmark de Performance (Extração Bruta)

### Teste de Geração de Thumbnails RAW (Modo Debug vs Release)

| Arquivo | Formato Origem | Tamanho (MB) | Tempo (Debug) | Tempo (Release) |
|---|---|---|---|---|
| RAW_CANON_DCS1.TIF | TIF | 6.1 MB | 857.109965ms | 3.805566ms |
| RAW_CANON_1DMARK3.CR2 | CR2 | 13.9 MB | 7.104237578s | 152.468793ms |
| sample1.nrw | NRW | 15.6 MB | 19.169519302s | 443.257216ms |
| RAW_MINOLTA_7D_SRGB.MRW | MRW | 8.8 MB | 638.583553ms | 35.738955ms |
| RAW_NIKON_P6000_GPS.NRW | NRW | 20.6 MB | 26.458640433s | 519.770884ms |
| Phase One - P45 - IIQ S (4_3).TIF | TIF | 28.6 MB | 7.433114178s | 3.89098ms |
| RAW_SONY_DSC-F828.SRF | SRF | 16.6 MB | 689.761177ms | 94.190265ms |
| Sony - DSC-F828 - 4_3.SRF | SRF | 16.6 MB | 712.934188ms | 6.354943ms |
| DSCF2121.RAF | RAF | 12.6 MB | 3.040603333s | 4.541533ms |
| P1000134.RW2 | RW2 | 29.2 MB | 4.668346726s | 9.373238ms |
| RAW_POLAROID__X530.X3F | X3F | 6.6 MB | 3.709110375s | 116.999742ms |
| RAW_PANASONIC_DMC-GF1.RW2 | RW2 | 14.0 MB | 7.069639948s | 5.412785ms |
| sample1.rw2 | RW2 | 16.7 MB | 7.499073676s | 5.807839ms |
| RAW_SAMSUNG_NX100.SRW | SRW | 25.9 MB | 31.022035404s | 32.863878ms |
| RAW_KODAK_DCS560C.tiff | TIFF | 5.3 MB | 348.471043ms | 281.014649ms |
| RAW_LEICA_DIGILUX2_SRGB.RAW | RAW | 9.4 MB | 4.478318087s | 2.174280205s |
| leica_d_lux_5_titanium_02.rwl | RWL | 12.3 MB | 5.931523223s | 8.577071ms |
| P1140545.RW2 | RW2 | 23.0 MB | 9.819276386s | 210.049028ms |
| RAW_KODAK_DCS460D_FILEVERSION_3.tiff | TIFF | 6.1 MB | 905.101906ms | 4.072798ms |
| Panasonic-Lumix-S1R-raw-00016.rw2 | RW2 | 66.6 MB | 7.755116146s | 79.666681ms |
| P1000101.RW2 | RW2 | 28.5 MB | 7.626311904s | 212.818681ms |
| sample1 (1).rw2 | RW2 | 16.7 MB | 7.685211793s | 3.758705ms |
| Leica - V-Lux 4 - RWL (4_3).RWL | RWL | 14.3 MB | 6.015195564s | 5.174955ms |
| sigma_sd_quattro_h_14.x3f | X3F | 65.1 MB | 97.567837794s | 2.433562167s |
| Panasonic-Lumix-S1R-raw-00001.rw2 | RW2 | 66.6 MB | 5.645301694s | 6.387895ms |
| RAW_PANASONIC_G1.RW2 | RW2 | 14.0 MB | 6.865778s | 368.414094ms |
| RAW_LEICA_DLUX3.RAW | RAW | 20.1 MB | 7.259193249s | 1.05545024s |
| SDIM0024.X3F | X3F | 57.3 MB | 223.035282513s | 2.723579536s |
| sigma_sd_quattro_h_03.x3f | X3F | 68.0 MB | 244.214880098s | 3.544208133s |
| RAW_POLAROID_X530.X3F | X3F | 6.6 MB | 12.899907991s | 176.985358ms |
| RAW_SIGMA_DP1.X3F | X3F | 15.5 MB | 34.299078373s | 323.575292ms |
| Sony - DSC-R1 - 14bit 14bit uncompressed (3_2).SR2 | SR2 | 20.1 MB | 3.341701376s | 51.712107ms |
| SDIM0023.X3F | X3F | 51.8 MB | 140.861747948s | 1.955393641s |
| RAW_HASSELBLAD_IXPRESS_CF132.3FR | 3FR | 32.0 MB | 18.906786741s | 316.328199ms |
| RAW_HASSELBLAD_CFV.3FR | 3FR | 21.8 MB | 13.546003707s | 234.289184ms |
| RAW_HASSELBLAD_H3D39II.3FR | 3FR | 52.5 MB | 17.80595938s | 544.911336ms |
| Hasselblad - X1D II 50C - 16bit (4_3).3FR | 3FR | 103.4 MB | 77.248925246s | 359.297875ms |
| P8160635.ORF | ORF | 16.6 MB | 56.99728385s | 175.758692ms |
| P8140131.ORF | ORF | 16.6 MB | 57.48430592s | 169.65061ms |
| RAW__KODAK_DC50.KDC | KDC | 0.1 MB | 750.066152ms | 56.572926ms |
| RAW__KODAK_EASYSHARE_Z1015-IS.KDC | KDC | 18.9 MB | 70.315621201s | 451.955496ms |
| RAW_NIKON_D5100.NEF | NEF | 15.5 MB | 92.709978547s | 419.884435ms |
| RAW_KODAK_DC120_WITH_JPEG.KDC | KDC | 0.1 MB | 777.839579ms | 28.926333ms |
| 3U2A67_32.CR3 | CR3 | 30.1 MB | 12.120441841s | 368.860445ms |
| Velvia_Prima.NEF | NEF | 10.7 MB | 70.04424163s | 277.660209ms |
| kodak_easyshare_z990_05.kdc | KDC | 21.6 MB | 70.819987805s | 626.62305ms |
| _SPC2147.NEF | NEF | 71.2 MB | 171.838518812s | 16.48081ms |
| RAW_NIKON_COOLPIX_P7100.NRW | NRW | 15.7 MB | 58.979538s | 308.064628ms |
| RAW_NIKON_D7000.NEF | NEF | 17.7 MB | 66.846332838s | 439.741193ms |
| Ritocco_Raw_Prima.NEF | NEF | 14.9 MB | 18.907506288s | 23.084496ms |
| RAW_NIKON_D800_12bit_FX_UNCOMPRESSED.NEF | NEF | 56.2 MB | 64.663389742s | 1.224797278s |
| 123A1863.CR3 | CR3 | 43.2 MB | 8.408857862s | 109.292482ms |
| RAW_NIKON_D3X.NEF | NEF | 27.3 MB | 60.410185741s | 510.000457ms |
| vladimirtalancev@vladhdv_dsc08557_174609464972.arw | ARW | 65.9 MB | 203.659639845s | 1.021477837s |
| RAW_SONY_DSC-RX100M2.ARW | ARW | 19.8 MB | 17.792672937s | 95.70343ms |
| DSC00131.ARW | ARW | 35.7 MB | 222.498882542s | 828.395261ms |
| RAW_SONY_A100.ARW | ARW | 8.7 MB | 4.5071598s | 33.837702ms |
| RAW_SONY_A700.ARW | ARW | 12.3 MB | 20.715801891s | 90.897207ms |
| stevensanchez@ssz_photos_ssz7911_174540096383.arw | ARW | 41.1 MB | 19.59329139s | 110.30418ms |
| RAW_SONY_RX10.ARW | ARW | 19.9 MB | 16.713611728s | 152.133821ms |
| RAW_SONY_NEX3.ARW | ARW | 14.0 MB | 12.000413755s | 55.208226ms |
| DSC00086.ARW | ARW | 35.0 MB | 222.947190772s | 1.578953213s |
| 163A9276.CR3 | CR3 | 31.7 MB | 13.400224694s | 117.926476ms |
| sample1.raf | RAF | 53.8 MB | 113.356901531s | 345.5026ms |
| Trittico_Prima01.NEF | NEF | 25.9 MB | 137.993716683s | 108.05056ms |
| DSCF2126.RAF | RAF | 12.6 MB | 7.959313238s | 2.910381ms |
| RAW_NIKON_D5000.NEF | NEF | 11.2 MB | 54.521903225s | 375.266243ms |
| RAW_NIKON_D800_12bit_FX_LOSSLESS.NEF | NEF | 32.8 MB | 68.086407191s | 866.232769ms |
| 163A8330.CR3 | CR3 | 29.1 MB | 2.287546843s | 76.080508ms |
| 3U2A6_708.CR3 | CR3 | 28.8 MB | 2.517160153s | 87.955762ms |
| 163A8322.CR3 | CR3 | 43.5 MB | 2.63093307s | 91.087207ms |
| RAW_FUJI_XQ1.RAF | RAF | 18.5 MB | 5.309197735s | 168.826482ms |
| RAW_CANON_10D.CRW | CRW | 6.1 MB | 9.196233076s | 407.186891ms |
| RAW_CANON_G5_SRGB.CRW | CRW | 4.2 MB | 492.624281ms | 76.062178ms |
| Leaf - Credo 40 - IIQ Sv2 (4_3).IIQ | IIQ | 21.1 MB | 2.906300075s | 178.970947ms |
| sample1.nef | NEF | 18.4 MB | 15.613784266s | 477.51378ms |
| example.raf | RAF | 48.2 MB | 4.767473188s | 69.711957ms |
| Phase One - P65+ - IIQ L (4_3) (1).iiq | IIQ | 60.9 MB | 5.790899265s | 432.56813ms |
| RAW_CANON_D60_ARGB.CRW | CRW | 6.3 MB | 5.815685281s | 254.230271ms |
| RAW_CANON_1DSM2.CR2 | CR2 | 14.2 MB | 2.926527381s | 78.782841ms |
| RAW_CANON_40D_RAW_V336643C.CR2 | CR2 | 12.6 MB | 4.245272359s | 146.495148ms |
| Hasselblad - H5D-40 - 16bit (4_3).fff | FFF | 66.6 MB | 6.187665717s | 1.052087641s |
| RAW_CANON_1DSM3.CR2 | CR2 | 19.9 MB | 15.125067444s | 188.465155ms |
| RAW_EPSON_RD1.ERF | ERF | 9.5 MB | 3.500604687s | 41.297017ms |
| RAW_MINOLTA_DIMAGE_A200.MRW | MRW | 11.6 MB | 2.709665078s | 27.082857ms |
| Hasselblad - Hasselblad H4D-40 - 16bit (4_3).fff | FFF | 69.8 MB | 25.387197038s | 1.1532191s |
| Phase One - P65+ - IIQ L (4_3).IIQ | IIQ | 68.6 MB | 18.842621393s | 444.049286ms |
| RAW_CANON_5DMARK2_PREPROD.CR2 | CR2 | 25.2 MB | 70.706794151s | 533.069588ms |
| sample1.dng | DNG | 6.1 MB | 1.561416532s | 64.78831ms |
| DJI-mavic-2-pro-raw-00009.dng | DNG | 39.6 MB | 1.683002563s | 63.968766ms |
| - credit_signatureedits.com - @thelo-10.dng | DNG | 8.6 MB | 9.489944936s | 156.27646ms |
| DJI-mavic-2-pro-raw-00006.dng | DNG | 39.5 MB | 1.259893366s | 160.043259ms |
| DJI-mavic-2-pro-raw-00012.dng | DNG | 39.2 MB | 1.138021872s | 74.456681ms |
| 800_0.0.DNG | DNG | 16.1 MB | 15.74063159s | 275.766961ms |
| RAW_LEICA_M240.DNG | DNG | 26.8 MB | 161.969676ms | 24.781651ms |
| RAW_PENTAX_K-R.DNG | DNG | 13.7 MB | 20.925495114s | 413.748363ms |
| RAW_PENTAX_K10D_SRGB.DNG | DNG | 16.2 MB | 17.486775846s | 298.687159ms |
| BF_00006.DNG | DNG | 40.3 MB | 43.015736806s | 865.014817ms |
| Hasselblad - CFV-50c - 16bit (4_3).fff | FFF | 75.9 MB | 5.213947142s | 724.906975ms |
| RAW_CANON_EOS_60D_V108_VERTICAL.CR2 | CR2 | 25.9 MB | 20.857452382s | 435.065552ms |
| RAW_MINOLTA_A1.MRW | MRW | 7.2 MB | 505.257647ms | 34.13815ms |
| Phase One - IQ3 100MP - Unknown (8) (4_3).IIQ | IIQ | 127.5 MB | 4.292467917s | 1.465955585s |
| RAW_CANON_EOS_7D.CR2 | CR2 | 22.1 MB | 21.058759421s | 961.571805ms |
| 0c0a0435.cr2 | CR2 | 26.1 MB | 76.189144611s | 667.288125ms |
| x1d-II-sample-01.fff | FFF | 77.8 MB | 4.398180345s | 547.487876ms |
| Phase One - iXU180 - IIQ Sv2 (4_3).IIQ | IIQ | 55.6 MB | 3.748361702s | 527.402806ms |
| sample_canon_400d1.cr2 | CR2 | 10.4 MB | 13.64505767s | 274.779577ms |
| RAW_CANON_EOS1200D.CR2 | CR2 | 20.5 MB | 22.207946236s | 446.338806ms |
| RAW_LEAF_APTUS22.MOS | MOS | 41.4 MB | 284.086964ms | 31.213492ms |
| RAW_PENTAX_K-m.PEF | PEF | 10.9 MB | 15.781270401s | 244.994569ms |
| RAW_CANON_50D.CR2 | CR2 | 18.1 MB | 23.796977909s | 478.020485ms |
| RAW_CANON_EOS_1DX.CR2 | CR2 | 24.9 MB | 24.074456624s | 1.050627771s |
| RAW_CANON_EOS_5DMARK3.CR2 | CR2 | 36.3 MB | 28.886384286s | 580.061454ms |
| RAW_CANON_5D_ARGB.CR2 | CR2 | 10.6 MB | 6.839419926s | 197.250157ms |
| sample_canon_350d_broken.cr2 | CR2 | 9.3 MB | 27.445806714s | 457.108492ms |
| sample1.pef | PEF | 20.1 MB | 37.686067924s | 330.068717ms |
| RAW_CANON_EOS_700D.CR2 | CR2 | 22.2 MB | 21.741675707s | 442.687657ms |
| RAW_KODAK_DC50.KDC | KDC | 0.1 MB | 125.601553ms | 18.895958ms |
| kodak_easyshare_z990_09.kdc | KDC | 21.3 MB | 15.131315644s | 323.312563ms |
| RAW_MAMIYA_ZDD.MEF | MEF | 34.9 MB | 3.632823147s | 554.662072ms |
| fujifilm_x_e1_13.raf | RAF | 24.9 MB | 3.463792411s | 126.418009ms |
| RAW_PENTAX_K-7.PEF | PEF | 14.6 MB | 41.294181475s | 292.089995ms |
| RAW_CANON_6D.CR2 | CR2 | 21.8 MB | 28.062826767s | 494.247601ms |
| RAW_CANON_EOS_5DS.CR2 | CR2 | 64.0 MB | 91.861506122s | 1.761197951s |
| RAW_PENTAX_KX.PEF | PEF | 10.0 MB | 15.820800176s | 275.433603ms |
| DSCF2125.RAF | RAF | 12.6 MB | 2.117378917s | 5.500955ms |
| RAW_FUJI_S5000.RAF | RAF | 6.5 MB | 1.800230037s | 77.728816ms |
| RAW_PENTAX_KD10.PEF | PEF | 9.8 MB | 14.180161311s | 204.174539ms |
| RAW_CANON_EOS-M3.CR2 | CR2 | 27.2 MB | 36.41829697s | 594.107066ms |
| sample1.cr2 | CR2 | 64.0 MB | 73.681340981s | 1.364806244s |
| RAW_OLYMPUS_E-PM1.ORF | ORF | 10.5 MB | 12.67632067s | 199.430703ms |
| P8120069.ORF | ORF | 16.8 MB | 12.798198233s | 177.065997ms |
| RAW_OLYMPUS_E1.ORF | ORF | 10.2 MB | 2.305371473s | 61.488995ms |
| RAW_OLYMPUS_E5.ORF | ORF | 12.2 MB | 10.394183266s | 192.667873ms |
| RAW_CANON_40D_SRAW_V103.CR2 | CR2 | 6.5 MB | 4.094110266s | 94.254197ms |
| RAW_CANON_EOS_1DM4.CR2 | CR2 | 18.6 MB | 26.305095562s | 405.246232ms |
| RAW_CANON_EOS70D.CR2 | CR2 | 22.7 MB | 34.121925624s | 450.41121ms |
| RAW_CANON_40D_RAW_V105.CR2 | CR2 | 11.8 MB | 5.241901322s | 125.08236ms |
