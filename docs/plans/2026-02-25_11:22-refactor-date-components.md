# Task: Refatoração de DateInput e DatePicker

Refatorar os componentes de data para seguir os padrões SOLID, melhorar a arquitetura modular, garantir conformidade com as diretrizes de documentação e nomenclatura, e integrar corretamente com o sistema de atalhos globais.

## 📋 Critérios de Aceite

- [x] Pastas dedicadas criadas: `src/components/ui/DateInput/` e `src/components/ui/DatePicker/`.
- [x] Componente `DateInput` utilizando `useInputEvents` para gerenciar o escopo de edição e atalhos.
- [x] Lógica de navegação do `DatePicker` extraída para um Hook customizado (`useDatePicker.ts`).
- [x] Nomes de variáveis descritivos (sem abreviações) e documentação TSDoc em inglês.
- [x] CSS colocado dentro das respectivas pastas.
- [x] Reaproveitamento das funções utilitárias de `src/utils/format.ts`.
- [x] Verificação de acessibilidade (navegação por teclado no input).

## 🔧 Ajustes Pós-Feedback (IDE)

- [x] **Complexidade**: Reduzida para < 10 em `DateInput.tsx` movendo helpers para o escopo do arquivo.
- [x] **Reatividade**: Corrigidos avisos em `useDatePicker.ts` garantindo que `props.value` seja acessado em escopo rastreado.
- [x] **Exportações**: Atualizado `ui/index.ts` para usar `export *` resolvendo problemas de resolução de tipos renomeados (`Properties`).
- [x] **Lint**: Removidos imports duplicados e espaços em branco excedentes.

## 🏗️ Estrutura Proposta

```text
src/components/ui/
├── DateInput/
│   ├── index.ts
│   ├── DateInput.tsx
│   ├── types.ts
│   └── date-input.css
└── DatePicker/
    ├── index.ts
    ├── Root.tsx
    ├── useDatePicker.ts
    ├── types.ts
    └── date-picker.css
```

## 🛠️ Plano de Ação

### Fase 1: Preparação e Tipagem
1. Criar as pastas `DateInput` e `DatePicker`.
2. Definir `types.ts` para ambos os componentes com documentação clara.

### Fase 2: Implementação do DatePicker
1. Implementar `useDatePicker.ts` para gerenciar o estado do calendário (dia, mês, ano, visualização ativa).
2. Criar `Root.tsx` (antigo `DatePicker.tsx`) refatorado.
3. Mover `date-picker.css` e atualizar importações.

### Fase 3: Implementação do DateInput
1. Criar `DateInput.tsx` integrando o hook `useInputEvents`.
2. Adaptar a lógica de máscara para ser mais limpa.
3. Mover `date-input.css` e atualizar importações.

### Fase 4: Integração e Limpeza
1. Atualizar importações em `DateCriterionField.tsx`.
2. Remover arquivos antigos `src/components/ui/DateInput.tsx` e `src/components/ui/DatePicker.tsx`.
3. Validar nomenclatura e documentação final.

## 🧪 Verificação
1. Testar seleção de data em `DateCriterionField`.
2. Verificar se atalhos globais (ex: 'S' para Search) são bloqueados enquanto o DateInput está focado.
3. Testar navegação por teclado no calendário.
