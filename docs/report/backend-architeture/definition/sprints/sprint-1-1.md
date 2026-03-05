# Sprint 1.1: Arcabouço Físico e Error Handling

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 1:** Fundação & Observabilidade (Core Mínimo)
**Objetivo:** Estabelecer a base física do repositório (diretórios Hexagonais) e a espinha dorsal de tratamento de erros. Ao fim desta sprint, o backend compilará com a nova estrutura e interceptará falhas de forma padronizada para o Frontend.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Compilação Limpa:** O projeto Tauri deve compilar normalmente (nenhum aviso grave no `cargo clippy`).
2. **Estrutura Criada:** As pastas `core`, `feature`, `infra` e `delivery` existem em `src-tauri/src/`.
3. **Payload de Erro Interceptado:** Um comando Tauri temporário de teste (Ex: `force_error`) ao ser acionado via Frontend, deve retornar o JSON no formato:
   ```json
   {
     "code": "INTERNAL_ERROR",
     "message": "Simulação de Falha de Banco",
     "details": "Conexão recusada"
   }
   ```

---

## 📋 Tarefas (Checklist do Agente)

### 1. Reestruturação de Diretórios
- [ ] Criar os diretórios base em `src-tauri/src/`:
  - `core/error`
  - `core/events`
  - `core/formats`
  - `feature/`
  - `infra/`
  - `delivery/tauri/`
- [ ] Atualizar o `main.rs` ou `lib.rs` para registrar e reconhecer as novas pastas como módulos públicos (`pub mod core;` etc).

### 2. Tratamento Centralizado de Erros (`AppError` & `AppResult`)
- [ ] Navegar ao arquivo base de erros (provavelmente reescrevendo/deslocando o atual `src/error.rs` para `src/core/error/mod.rs`).
- [ ] Atualizar o enum `AppError` com base no `thiserror` contendo pelomenos as ramificações primordiais:
  - `DatabaseQueryFailed(String)`
  - `FileSystem(std::io::Error)`
  - `FormatNotSupported(String)`
  - `ExtractionProcessTimeout`
- [ ] Implementar a trait `serde::Serialize` ativamente no `AppError` para retornar estruturalmente o `code` genérico da falha + `message` humanizada, blindando o Frontend contra logs obscuros do SQLx.
- [ ] Exportar o alias `pub type AppResult<T> = Result<T, AppError>;` para onipresença no repositório.

### 3. Log e Telemetria (Tracing)
- [ ] Validar se as dependências do `tracing` e `tracing-subscriber` estão presentes no `Cargo.toml`.
- [ ] Em `main.rs` (Bootloader), abstrair a configuração num `init_telemetry()`. Definir para interceptar LOG `info` ou `debug` formatado no terminal com timestamps.

### 4. Teste de Ponto a Ponto (Mock Command)
- [ ] Em `delivery/tauri/mod.rs`, criar um comando de teste `#[tauri::command] pub async fn test_error(fail: bool) -> AppResult<String>`.
- [ ] Fazer a injeção via Builder do Tauri: `.invoke_handler(tauri::generate_handler![test_error])`.
- [ ] Acionar tal comando via Javascript/Typescript do App e conferir se o Painel DevTools recebe exatamente a classe tratada `AppError` convertida, cravando a fundação limpa do sistema.

---

## 💡 Notas para o Desenvolvedor / Agente
> O escopo desta Sprint é intencionalmente isolado de Banco de Dados ou Regras de Negócio. Mantenha os PRs focados somente nas ramificações base para não afogar o contexto. Utilize extensivamente o `tracing::debug!` durante a simulação do Command.
