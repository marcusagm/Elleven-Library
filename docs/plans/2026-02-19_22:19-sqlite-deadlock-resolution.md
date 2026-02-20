# Resolução de Deadlocks no SQLite (Database is locked)

## 📌 Contexto

Ao iniciar a aplicação com um banco de dados vazio e adicionar uma pasta com muitas imagens ("Location"), notamos que muitos arquivos deixavam de ser indexados, e o terminal apontava falhas consistentes de salvamento:
`Failed to save image in batch: error returned from database: (code: 5) database is locked`.

Esse problema afetava gravemente a confiabilidade do indexador em lote.

---

## 🔍 Análise do Problema

O Mundam utiliza SQLite no modo **WAL** (Write-Ahead Logging), o qual permite múltiplas leituras simultâneas e uma escrita por vez de forma concorrente. No ecossistema assíncrono em Rust usando `slqx`:
1. Quando chamamos `pool.begin().await?`, o SQLite inicia por padrão uma transação **DEFERRED** (adiada).
2. Na transação adiada, é emitido apenas um *Shared Lock* (Leitura).
3. O indexador paraleliza o processamento e várias "threads" assíncronas começam suas transações dessa mesma exata maneira.
4. Ao tentar um `INSERT` ou `UPDATE` mais tarde no bloco de código, a conexão pede para escalar seu acesso para um *Exclusive Lock* (Gravação).
5. Como múltiplas threads possuem o lock de leitura compartilhado (devido às seleções antes da hora de escrever), o SQLite entra em um impasse: nenhuma cede a leitura para a outra gravar. Isso é um **Deadlock** clássico de bancos paralelos.
6. O SQLite responde matando o pedido de escrita com `(code 5) database is locked`.

---

## 🛠 Passos da Solução Implementada

Para resolver o problema sem prejudicar a leitura concorrente do painel UI, configuramos as opções nativas e forçamos aquisições proativas de bloqueio.

### 1. Ajuste do Timeout via Driver `SqliteConnectOptions`
Em `src-tauri/src/db/mod.rs`, alteramos a construção do pool para definir explicitamente:
- `journal_mode(SqliteJournalMode::Wal)`
- `synchronous(SqliteSynchronous::Normal)`
- `busy_timeout(std::time::Duration::from_secs(30))`

Isso instrui o SQLite a re-tentar amigavelmente a conexão de escrita se ela estiver bloqueada, esperando até 30 segundos no caso de operações pesadas, antes de abortar. As chamadas cruas tipo `PRAGMA` após a inicialização do pool foram removidas para prevenir dessincronização do estado em cache do `sqlx`.

### 2. Lock Exclusivo Imediato (BEGIN IMMEDIATE *Mock*)
Como a interface do driver `sqlx` (ainda) não disponibiliza um método padronizado para chamar `BEGIN IMMEDIATE;` entre as apis rust, adicionamos uma manobra estruturada.

Nos lotes de escrita das entidades, especificamente em:
- `src-tauri/src/db/images.rs` (`save_images_batch`)
- `src-tauri/src/db/folders.rs` (`ensure_folder_hierarchy`)
- `src-tauri/src/db/tags.rs` (`add_tags_to_images_batch`)

Imediatamente após iniciar a transação (`let mut tx = self.pool.begin().await?`), introduzimos uma consulta no banco inofensiva e controlada, forçando a escrita:
```rust
sqlx::query("INSERT INTO app_settings (key, value) VALUES ('_db_lock', '1') ON CONFLICT(key) DO UPDATE SET value = '1'")
    .execute(&mut *tx)
    .await
    .ok();
```
Isso força o SQLite a escalar a transação recém nascida para gravação de antemão. Consequentemente, se outra thread tentar abrir um lote, o timeout de 30s as organiza harmoniosamente num modelo em "fila", mitigando completamente os colisões.

---

## 🧱 Obstáculos Encontrados

- A principal limitação imposta por bibliotecas ORM ou Construtores de sintaxe é a ausência de recursos finos nativos de motores. No caso, não possuíamos uma API simples em Rust como `begin_write()` ou algo análogo pelo SQLx. 
- Sem o artifício da injeção explícita de operação mock numa tabela em background (`app_settings`), forçar as operações em lote envolveria criar um Lock global com o `std::sync::Mutex` em Rust isolando essas rotinas na memória, o que seria menos expressivo e traria complexidade assíncrona extra à gestão de *State* através do AppHandle do Tauri.

---

## 🚀 Melhorias Futuras e Considerações

1. **Monitoramento e Aprimoramento do SQLX:** Estudar uma forma de passar strings explícitas à api para evitar requisições nulas ao banco. Caso o próprio SQLX passe a suportar o statement `BEGIN IMMEDIATE`, remover o mock do pacote.
2. **Channel-Based Batching (Fila Assíncrona Rust):** Como o indexador se tornará futuramente responsável por bibliotecas com até milhares de arquivos paralelos, escalar o sistema mudando-o de: _Vários batches pequenos competindo por lock_ para _Criação de 1 Thread "Escritora" Singleton conectada via um canal `mpsc` ao coletor. O coletor vai lendo tudo e somente a thread master salva assincronamente.
3. **Resolução de Logs do FFmpeg:** Os últimos processos da UI renderizaram no terminal avisos como `FFmpeg failed (segment 0) [...] Output file does not contain any stream`. Pode ser necessário revisar em seguida a extração e fallback de vídeos em lote para o `PreviewExtractor`.
