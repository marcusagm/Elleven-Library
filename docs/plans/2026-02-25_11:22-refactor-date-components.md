# Task: Refatoração de DateInput e DatePicker

Refatorar os componentes de data para seguir os padrões SOLID, melhorar a arquitetura modular, garantir conformidade com as diretrizes de documentação e nomenclatura, e integrar corretamente com o sistema de atalhos globais.

## 📋 Critérios de Aceite

- [x] Pastas dedicadas criadas: `src/components/ui/DateInput/` e `src/components/ui/DatePicker/`.
- [x] Componente `DateInput` utilizando `useInputEvents` para gerenciar o escopo de edição e atalhos.
- [x] Lógica de navegação do `DatePicker` extraída para um Hook customizado (`useDatePicker.ts`).
- [x] Nomes de variáveis descritivos (sem abreviações) e documentação TSDoc em inglês.
- [x] CSS colocado dentro das respectivas pastas.
- [x] Reaproveitamento das funções utilitárias de `src/utils/format.ts`.
- [x] Centralização de utilitários de data em `src/utils/format.ts` (parsing e formatação DD/MM/YYYY).
- [x] Correção da exibição de intervalos de data na Busca Avançada.

## 🔧 Ajustes Pós-Feedback (IDE)

- [x] **Complexidade**: Reduzida para < 10 em `DateInput.tsx` movendo helpers para o escopo do arquivo.
- [x] **Reatividade**: Corrigidos avisos em `useDatePicker.ts` garantindo que `props.value` seja acessado em escopo rastreado.
- [x] **Exportações**: Atualizado `ui/index.ts` para usar `export *` resolvendo problemas de resolução de tipos renomeados (`Properties`).
- [x] **Lint**: Removidos imports duplicados e espaços em branco excedentes.
- [x] **Robustez**: Função `formatToDisplay` atualizada para aceitar tanto strings ISO quanto objetos `Date`, evitando erros de renderização no SolidJS.

## 🏗️ Estrutura Finalizada

```text
src/
├── components/ui/
│   ├── DateInput/
│   │   ├── index.ts
│   │   ├── DateInput.tsx [Refatorado + useInputEvents]
│   │   ├── types.ts
│   │   └── date-input.css
│   └── DatePicker/
│       ├── index.ts
│       ├── Root.tsx [Componente Visual]
│       ├── useDatePicker.ts [Lógica de Estado]
│       ├── types.ts
│       └── date-picker.css
└── utils/
    └── format.ts [Central de Utilitários de Data]
```

## 🛠️ Plano de Ação (Executado)

### Fase 1: Preparação e Tipagem (Concluído)
1. Criar as pastas `DateInput` e `DatePicker`.
2. Definir `types.ts` para ambos os componentes com documentação clara.

### Fase 2: Implementação do DatePicker (Concluído)
1. Implementar `useDatePicker.ts` para gerenciar o estado do calendário (dia, mês, ano, visualização ativa).
2. Criar `Root.tsx` refatorado.

### Fase 3: Implementação do DateInput (Concluído)
1. Criar `DateInput.tsx` integrando o hook `useInputEvents`.
2. Adaptar a lógica de máscara para ser mais limpa.

### Fase 4: Centralização e Refinamento (Concluído)
1. Mover `formatDateToDisplay` e `parseDisplayDate` para `src/utils/format.ts`.
2. Implementar `formatDisplay` no `DateCriterionField.tsx` para exibição correta de critérios de busca.
3. Atualizar infraestrutura de busca (`searchHelpers.ts`, `useAdvancedSearch.ts`) para suportar exibição de intervalos (Data 1 "to" Data 2).
4. Sincronizar todos os handlers de busca (`size`, `folder`, `rating`, `tags`) com a nova assinatura do `formatDisplay`.

### Fase 5: Padronização e Documentação (Concluído)
1. Revisar todos os arquivos em conformidade com `@frontend-solid.md` e `@documentation.md`.
2. Renomear variáveis abreviadas para nomes descritivos (ex: `props` -> `properties`, `local` -> `localProperties`).
3. Adicionar TSDocs extremamente detalhados em todos os componentes e utilitários.

## 🧪 Verificação Final
1. [x] Testar seleção de data em `DateCriterionField`.
2. [x] Verificar se atalhos globais (ex: 'S' para Search) são bloqueados enquanto o `DateInput` está focado.
3. [x] Validar que intervalos de data aparecem corretamente na barra de busca (ex: "01/02/2026 to 25/02/2026").
4. [x] Confirmar ausência de erros de renderização [Unrecognized value] no console.
