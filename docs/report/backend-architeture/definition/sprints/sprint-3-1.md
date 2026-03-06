# Sprint 3.1: Interface FormatRegistry & Estrutura Base

**Status:** ✅ Concluído
**Data e hora de inicio:** 2026-03-06T16:58:22
**Data da conclusão:** 2026-03-06T17:35:00

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Estabelecer o roteador O(1) de formatos e definir as Traits passivas (`MetadataCapability`, `ThumbnailCapability`). Esta sprint não converte nenhuma imagem, apenas fia a "placa-mãe" onde os extratores (plugins) se conectarão.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. [x] **Estruturas O(1) Operacionais:** O `FormatRegistry` deve instanciar e manter o `HashMap` indexando extensões corretamente (ignora *case sensitivity*).
2. [x] **Resolução Rápida (By Extension):** Pedir suporte para "arquivo.psD" devolve instantaneamente um provedor Mokado cadastrado, sem iterar arrays.
3. [x] **Resolução Profunda (Magic Bytes):** Pedir suporte para um arquivo sem extensão, com *header* `[0x89, 0x50, 0x4E, 0x47]` devolve Mock do Extrator PNG pela rota secundária (N de complexidade).

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição das Capabilities
- [x] Em `src-tauri/src/core/formats/capabilities.rs`, portar a trait `#[async_trait] pub trait MetadataCapability` com os métodos técnicos/semânticos.
- [x] Portar a trait `#[async_trait] pub trait ThumbnailCapability` (assinatura de array de bytes de memória + request de size limit).

### 2. O FormatProvider Principal
- [x] Em `core/formats/provider.rs`, fundar a trait `FormatProvider` com os 5 métodos (Nome, vetor com extensões aceitas `supported_extensions()`, `supports_magic_bytes()` falso por default, e os 2 getters lógicos usando `Option<&dyn>`).

### 3. Implementação do Roteador (Registry)
- [x] Em `core/formats/registry.rs`, montar a estrutura `FormatRegistry` com o `HashMap<String, Arc<dyn FormatProvider>>` (Cache) e o `Vec` passivo (Fallbacks).
- [x] Executar o método de Cadastro (`register()`) auto-preenchendo as rotas do mapa.
- [x] Escrever o `fn resolve()` como coração técnico: Checar a Hash > Acionar Fallback via `.find()` se o File IO retornar que o OS informou uma Extensão fantasma ou nula.

### 4. Ponto Inicial (Bootlace) do Main
- [x] Escrever a factory `build_format_registry()` contendo o preenchimento bruto durante a carga local do container do Backend. Injetá-la no Tauri.

---

## 💡 Notas para o Desenvolvedor / Agente
> Você não vai instanciar `image-rs`, FFmpeg ou CLI aqui. Esta sprint apenas fornece a Cesta de Encaixe Hexagonal (As "Ports") baseadas firmemente no documento `format-implementation-guide.md`. Somente realize Testes Unitários estáticos criando Pseudo-Providers que dão PrintLn ao serem engatilhados perante chamadas de extensões soltas.

---

## 🛠️ Informações da Implementação

### Melhorias Realizadas
- **Concorrência e Segurança**: Implementação do `FormatRegistry` utilizando `Arc<dyn FormatProvider>`, garantindo que o registro de formatos seja Thread-Safe e compatível com o ambiente assíncrono do Tauri/Tokio.
- **Ergonomia**: Adicionada a trait `Default` para o `FormatRegistry` e uma função factory `build_format_registry` centralizada em `mod.rs`.
- **Testes Unitários**: Criados testes abrangentes que validam a resolução O(1), a insensibilidade a maiúsculas/minúsculas e o fallback de Magic Bytes.

### Dificuldades e Ajustes
- Durante a implementação, foram corrigidos avisos de `unused_imports` e mutabilidade excessiva em `registry.rs` e `mod.rs`, garantindo adesão plena ao "Clean Code" e às diretrizes do projeto.

---

## 📂 Arquivos Modificados / Criados

- [capabilities.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/capabilities.rs) (Criado)
- [provider.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/provider.rs) (Criado)
- [registry.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/registry.rs) (Criado)
- [mod.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/mod.rs) (Modificado)
- [lib.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/lib.rs) (Modificado)
