# 04. Unified Error Handling (Erros de Domínio Centralizados)

## 1. Visão Geral e Objetivo Macro

O **Sistema Centralizado de Erros** é a "rede de segurança invisível" da Arquitetura Hexagonal. Em projetos Rust comuns, é frequente ver funções retornando genéricos abusivos (ex: `anyhow::Result<()>`) ou dezenas de bibliotecas externas estourando erros que vazam direto para o Frontend (ex: *`sqlite error: code 5 locked`*). Isso não ajuda o Solid.js a desenhar uma UI amigável e compromete a segurança ao expor infraestrutura bruta.

O objetivo do módulo `core/error/` é servir como o "Tradutor Universal". Ele encapsula todos os erros de baixo nível da aplicação (do Banco, do FFmpeg, do FileSystem, Assinaturas Expiradas, etc.) em uma Coleção de **Erros de Domínio** significativos e controlados. Ele define estritamente o `AppResult<T>` global e garante que *todo* pânico ou falha previsível do Backend se converta em um JSON civilizado quando cruzar a fronteira Tauri IPC rumo à Interface Gráfica.

## 2. Localização Exata
`src-tauri/src/core/error/`
- Arquivos prováveis: `mod.rs` (O Root), `domain.rs` (O Enum Base de Erros) e `tauri_mapper.rs` (Implementação do trait `Serialize` para o IPC).

---

## 3. Responsabilidades

### O que NÓS FAZEMOS:
- **Tradução (*Mapping*):** Pegamos falhas complexas de adaptadores externos (ex: `sqlx::Error::Database` ou `std::io::Error::NotFound`) e convertemos em falhas ricas do Domínio (ex: `AppError::StorageLockTimeout` ou `AppError::AssetNotFound`).
- **Definição de API Central:** Fornecemos a única assinatura permitida para funções expostas do Core (`pub type AppResult<T> = Result<T, AppError>;`).
- **Formatação (Serialization) para a UI:** Implementamos o Trait `serde::Serialize` sobre nosso Enum genérico para cuspir objetos padronizados, com códigos de string curtos (como `RESOURCE_NOT_FOUND`) e "mensagens de usuário" consumíveis diretamente pelo Toast do UI (Solid.js).

### O que NÓS NÃO FAZEMOS:
- **O Módulo de Erros NÃO lida com "Fallback" Lógico:** Se um PDF der erro ao renderizar, quem decide colocar a imagem "padrão_quebrado.png" é o `FormatProvider` de PDF, *dentro* do seu limite. O Módulo Core de Errors apenas *fornece a classe do erro* que o provedor emite.
- **NÃO Substituímos o Logging de Telemetria (Tracing):** O erro em si *pode* ser loggado pelo `infra/telemetry`, mas esta pasta `error/` foca nas Estruturas de Dados do contrato, e não no ato de imprimir `tracing::error!()` na tela preta.

---

## 4. Diagrama de Fluxo e Tradução de Retorno (Boundary)

```mermaid
graph TD
    classDef infra fill:#fcf8e3,stroke:#f0ad4e,stroke-width:2px;
    classDef core fill:#d6f5d6,stroke:#5cb85c,stroke-width:2px;
    classDef ext fill:#ffcdd2,stroke:#f44336,stroke-width:2px;
    classDef ui fill:#f3e5f5,stroke:#8e24aa,stroke-width:2px;

    %% Acontecimentos de Baixo Nível
    FFMPEG(FFmpeg Process\nExitCode: 1):::infra
    SQLITE(SQLx Query\nRowNotFound):::infra
    FS(S.O. FileSystem\nAccessDenied):::infra

    %% Mapeamento Nativo (Rust Libraries)
    anyhow[std::io::Error\nsqlx::Error]:::ext

    FFMPEG -. falha .-> anyhow
    SQLITE -. panica .-> anyhow
    FS -. recusa .-> anyhow

    %% A Borda de Tradução (Nosso Módulo)
    ERR_MOD{core/error/domain.rs\nimpl From T for AppError}:::core

    anyhow -- Tradução Automática (From) --> ERR_MOD

    %% Saída do Domínio
    ERR_MOD -- Produz --> APP_ERR(AppError Enum\nAppError::Database, AppError::FormatProcessTimeout):::core

    %% Conversão para Transporte
    IPC_BOUNDARY[/Tauri #[command] Boundary\]
    JSON_SERIALIZATION(impl Serialize for AppError\nJSON Translator):::core

    APP_ERR --> IPC_BOUNDARY
    IPC_BOUNDARY --> JSON_SERIALIZATION

    %% Reação da UI
    SOLID([Solid.js UI\ni18n Toast Error]):::ui
    
    JSON_SERIALIZATION -- {code: OS_ACCESS_DENIED,\n message: ...} --> SOLID
```

---

## 5. Estruturas de Dados e Traits (O Contrato "Port")

O código-base em Rust dita um Enum gordinho, com variantes ricas apoiado via biblioteca `thiserror`. Isso elimina a fadiga mecânica de implementar display para cada vertente de falha.

```rust
// core/error/domain.rs
use thiserror::Error;
use serde::{Serialize, Serializer};

// A "Super-Piteira" Global das funções
pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    // ---- Erros de Máquina (Infra/BD) ----
    #[error("Database constraint or transaction violation: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Failed to access physical storage file: {0}")]
    Io(#[from] std::io::Error),
    
    // ---- Erros de Negócio Puros (O Asset Ledger usa isso) ----
    #[error("The requested Asset UUID '{0}' does not exist.")]
    AssetNotFound(String),
    
    #[error("Cannot transition Asset from '{from:?}' to '{to:?}'. Illegal Operation.")]
    IllegalStateTransition { from: String, to: String },
    
    #[error("Invalid Input Data during command creation: {0}")]
    ValidationFailed(String),
    
    // ---- Erros de Operários (Format/FFmpeg/Workers) ----
    #[error("Timeout while attempting to extract capability via Subprocess.")]
    ExtractionProcessTimeout,
    
    #[error("File format signature '{0}' is not supported by any known Capability Provider.")]
    FormatNotSupported(String),
}
```

O Mapeador Serde para a UI (Saindo pela Fronteira IPC):

```rust
// core/error/tauri_mapper.rs
// Transforma os enums complexos do Rust em Objetos JSON consumíveis em TypeScript

#[derive(Serialize)]
struct ErrorPayload {
    code: String,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (code, message) = match self {
            AppError::AssetNotFound(id) => (
                "NOT_FOUND",
                format!("Asset {} not found in library.", id),
            ),
            AppError::Database(_) => (
                "DATABASE_ERROR",
                "A strict database violation occurred.".to_string(), // Esconde do TS o DB Stack Trace
            ),
            AppError::ValidationFailed(reason) => (
                "VALIDATION_FAILED", 
                reason.to_string()
            ),
            // ... Mapeia restritivamente
            _ => (
                "INTERNAL_ERROR",
                self.to_string(),
            )
        };

        ErrorPayload {
            code: code.to_string(),
            message,
        }.serialize(serializer)
    }
}
```

---

## 6. Dependências Mútuas na Prática

1. **Quem Depende deste Módulo:** *"Todo Mundo"*. Todo arquivo Rust em `core/`, `feature/`, `processing/` vai começar o arquivo com `use crate::core::error::{AppResult, AppError};`. É a dependência mais central da árvore (deposta talvez por tipos de dados).
2. **Dependência Crítica no Tauri:** Os arquivos em `delivery/tauri/mod.rs` que abrigam os `#[tauri::command]` devem usar o `AppResult<T>`. Se eles esquecerem, os comandos do Tauri "panicam" explodindo o aplicativo em vez de falharem com dignidade mandando promessas rejeitadas para o UI.

---

## 7. Tratamento de Erros Esperados... Sobre o Próprio Erro

### **Cenário Único: A Mascarada de Segurança ("Information Leakage")**
- *Causa:* Um erro de SQLx injetou uma string nativa no `From<T>` que contém o IP ou a senha dura do Banco de Dados local (raro no SQLite, mas ocorre em PostgreSQL).
- *Comportamento do AppError:* O "Mapeador de Tauri" `(impl Serialize)` corta cirurgicamente o erro real fora do JSON. Ele transforma *todos* os vazamentos técnicos pesados de Infra na Genérica string: *"A strict database violation occurred"*, passando apenas a tag `"DATABASE_ERROR"` para o frontend e deixando o `tracing` registrar o vazamento feio somente nos logs salvos no computador (no terminal cego), preservando as diretrizes de código limpo e seguro.
