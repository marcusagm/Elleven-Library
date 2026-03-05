# Sprint 3.1: Interface FormatRegistry & Estrutura Base

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Estabelecer o roteador O(1) de formatos e definir as Traits passivas (`MetadataCapability`, `ThumbnailCapability`). Esta sprint não converte nenhuma imagem, apenas fia a "placa-mãe" onde os extratores (plugins) se conectarão.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Estruturas O(1) Operacionais:** O `FormatRegistry` deve instanciar e manter o `HashMap` indexando extensões corretamente (ignora *case sensitivity*).
2. **Resolução Rápida (By Extension):** Pedir suporte para "arquivo.psD" devolve instantaneamente um provedor Mokado cadastrado, sem iterar arrays.
3. **Resolução Profunda (Magic Bytes):** Pedir suporte para um arquivo sem extensão, com *header* `[0x89, 0x50, 0x4E, 0x47]` devolve Mock do Extrator PNG pela rota secundária (N de complexidade).

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição das Capabilities
- [ ] Em `src-tauri/src/core/formats/capabilities.rs`, portar a trait `#[async_trait] pub trait MetadataCapability` com os métodos técnicos/semânticos.
- [ ] Portar a trait `#[async_trait] pub trait ThumbnailCapability` (assinatura de array de bytes de memória + request de size limit).

### 2. O FormatProvider Principal
- [ ] Em `core/formats/provider.rs`, fundar a trait `FormatProvider` com os 5 métodos (Nome, vetor com extensões aceitas `supported_extensions()`, `supports_magic_bytes()` falso por default, e os 2 getters lógicos usando `Option<&dyn>`).

### 3. Implementação do Roteador (Registry)
- [ ] Em `core/formats/registry.rs`, montar a estrutura `FormatRegistry` com o `HashMap<String, Arc<dyn FormatProvider>>` (Cache) e o `Vec` passivo (Fallbacks).
- [ ] Executar o método de Cadastro (`register()`) auto-preenchendo as rotas do mapa.
- [ ] Escrever o `fn resolve()` como coração técnico: Checar a Hash > Acionar Fallback via `.find()` se o File IO retornar que o OS informou uma Extensão fantasma ou nula.

### 4. Ponto Inicial (Bootlace) do Main
- [ ] Escrever a factory `build_format_registry()` contendo o preenchimento bruto durante a carga local do container do Backend. Injetá-la no Tauri.

---

## 💡 Notas para o Desenvolvedor / Agente
> Você não vai instanciar `image-rs`, FFmpeg ou CLI aqui. Esta sprint apenas fornece a Cesta de Encaixe Hexagonal (As "Ports") baseadas firmemente no documento `format-implementation-guide.md`. Somente realize Testes Unitários estáticos criando Pseudo-Providers que dão PrintLn ao serem engatilhados perante chamadas de extensões soltas.
