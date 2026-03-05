# Sprint 4.3: Extração de Cores e Semântica (Análise Background)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 4:** O Músculo Operacional (Workflows) 
**Objetivo:** Restaurar a super feature local do Mundam: o poder de buscar arquivos pelas nuances da Cor Hexadecimal. Implementado como Workflow Estrito do Event-Driven System, ele reagirá aos rastros deixados pelos geradores de Thumbs no EventBus.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Reatividade Orgânica:** Sempre que o `JobScheduler` terminar uma Thumbnail com louvor, o sistema `ColorWorker` entra em ação sem que ninguém o force diretamente através de Código Síncrono, lendo o JSON que ecoou no Tópico Broadcast.
2. **K-Means / Extração Quantificada:** Utilizar algoritmos rápidos rodando exclusivamente sobre a Cópia-Thumbnail da imagem (pequena/WebP/JPG) para abduzir as frequências dominantes RGB.
3. **Armazenamento Seguro:** Conversão autônoma de RGB para valores CIELAB Euclidianos e gravação assíncrona contra a Tabela Relacionamental `AssetColor` e atrelando Cor Principal na coluna direta do Data Model original no CQRS via Adapter.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Escuta Genuína (EventBus)
- [ ] Em `infra/workers/color_worker.rs`, orquestrar subscrição direta do Tópico de Eventos Criado lá na Fase 1. Escutar `DomainEvent::ThumbGenerated`.

### 2. O Algoritmo K-Means Local
- [ ] No `feature/analysis/colors.rs` implementar ou transpor os modelos matemáticos pre-existentes no Mundam (`extract_color_palette`). Importante assegurar que ele atue abrindo a *Thumbnail Média Limpa* que já existe no Cache no SSD. 
- [ ] Retornar o pacote DTO bruto e transformável da *Cor Predominante*, além de Vetor com Top 5 cores ranqueadas (LAB % percentual).

### 3. Reconciliação Perfeita (CQRS Command)
- [ ] Tendo o Vetor DTO mapeado, disparamos a Inteção final: Construção do pacote imutável `UpdateAssetColorsCommand` e disparo direto do *Worker* contra a barreira do `AssetLedger`, que realizará Insert Atomático (Rollback protected) encurralando a inteligência extraída ao Banco.

### 4. Estabilidade do Loop Rayon
- [ ] A matemática visual é dispendiosa. O Worker que ouve o Bus deve encapsular o iterador visual usando as amarras MultiThreading do `rayon` dentro da porta assíncrona `spawn_blocking`, evitando qualquer soluço na concorrência da App global.

---

## 💡 Notas para o Desenvolvedor / Agente
> Esta Sprint representa a glória da EDA (Arquitetura Orentiada A Eventos). Na arquitetura legada, muitas conversões ocorrem procedurais. Sob este novo escopo, o Ledger nem sabe que a Cor Dominante foi procurada; o Worker trabalha solto. Foque brutalmente na matemática da distância Euclidiana nas planilhas: LAB conversions devem bater os mesmos espectros gerados na API do Typescript. Não abra o RAW/PSD de 1GB para pegar as Core! Extraia exclusivamente acessando a Imagem Minificada!
