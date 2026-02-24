# Arquitetura de Registro de Componentes Dinâmicos para Busca Avançada

## 1. Visão Geral

Este documento define o plano arquitetural para refatorar os componentes de interface de usuário envolvidos na definição de queries de busca avançada: `CriteriaBuilder.tsx` (responsável pela criação de novos critérios) e `QueryEditor.tsx` (responsável pela listagem e edição de critérios existentes).

O objetivo é substituir a grande quantidade de blocos condicionais de renderização (`<Show>`) baseados no tipo do campo de busca por um **Padrão de Registro de Componentes (Plugin Architecture / Component Registry)**. Isso proporcionará extrema testabilidade, separação de responsabilidades e aderência absoluta ao Princípio Aberto/Fechado (Open/Closed Principle - OCP).

## 2. Contexto e Problema Atual

Atualmente, `CriteriaBuilder.tsx` e `QueryEditor.tsx` sofrem com código inflado devido ao acoplamento excessivo das lógicas de UI e conversão de dados. Adicionar um novo tipo de busca exige modificar estes arquivos centrais, o que quebra o OCP.

Além disso, a injeção da tipagem complexa gerada pelo estado global para lidar com Range de Datas e Unidades de Tamanho ocorre no meio das renderizações de lista, gerando ruído e potenciais bugs de reatividade.

## 3. Arquitetura Proposta (Option B: Component Registry)

A solução baseia-se em desacoplar a renderização de cada tipo de campo (`date`, `size`, `folder`, `tags`, `string`) em seus próprios micro-componentes, isolando o estado visual e a conversão de seus valores primários. Estes micro-componentes serão catalogados em um dicionário estático chamado de "Registro".

### 3.1. Tipagem Compartilhada (Interfaces)

Todo componente de critério registrado deve respeitar a mesma assinatura de contrato (`Props`). Este contrato define o que os adaptadores visuais necessitam da interface de estado global (como `useAdvancedSearch`).

```tsx
// Exemplo de Proposta de Tipagem
export interface CriterionFieldRendererProps {
    fieldKey: string;
    operator: string;
    value: SearchValue | SearchValue[];
    onChangeValue: (value: SearchValue | SearchValue[]) => void;
    
    // Parâmetros opcionais auxiliares dependentes do campo
    unitMultiplier?: string;
    onChangeUnit?: (unit: string) => void;
    metadataContext?: MetadataContextType; // Para ler locations/tags se necessário
}
```

*Nota em concordância com Solid.js Guidelines*: Nunca iremos desestruturar esta interface (`Props`) no corpo do componente sem utilizar `splitProps`.

### 3.2. Micro-Componentes Específicos (Plugins)

Cada tipo de campo possuirá um componente próprio. Exemplos de arquivos dedicados a compor o módulo:

-   `DateCriterionField.tsx`
-   `SizeCriterionField.tsx`
-   `StringCriterionField.tsx`
-   `SelectCriterionField.tsx` (para Tags e Folders, possivelmente reutilizável)

Esses componentes lidarão internamente com suas nuances. O `SizeCriterionField`, por exemplo, gerenciará o seletor de input da quantia agregando o dropdown das unidades em um só lugar. O `DateCriterionField` lidará diretamente com a condicional de _"Se o operator for between, mostra dois <DateInput>"_, varrendo essa lógica espaguete da fachada central.

### 3.3. O Dicionário de Registro Central

Um mapa resolverá dinamicamente a instanciação baseada na definição global de configuração de mapeamento de tipos, localizada possivelmente em `searchConstants.ts` ou num arquivo `CriterionFieldRegistry.tsx`.

```tsx
import { DateCriterionField } from './fields/DateCriterionField';
import { SizeCriterionField } from './fields/SizeCriterionField';
// ...

export const criterionFieldRegistry: Record<string, Component<CriterionFieldRendererProps>> = {
    'date': DateCriterionField,
    'size': SizeCriterionField,
    // fallback genérico para textos
    'string': StringCriterionField, 
};
```

### 3.4. Consumo nos Arquivos Centrais

O `CriteriaBuilder.tsx` e `QueryEditor.tsx` passarão a funcionar como adaptadores agnósticos. Eles apenas coletarão o `type` (via lookup do Metadata Field) e invocarão o Dynamic Component.

```tsx
// Exemplo Conceitual no Iterador do QueryEditor
<For each={search.criteria()}>
    {(criterion) => {
        const fieldDefinition = getFieldDefinition(criterion.key);
        const DynamicFieldComponent = criterionFieldRegistry[fieldDefinition.type] || StringCriterionField;
        
        return (
             <div class="criterion-item">
                 {/* ... renderiza metadados: key, operator ... */}
                 
                 <DynamicFieldComponent 
                      fieldKey={criterion.key}
                      operator={criterion.operator}
                      value={criterion.value}
                      onChangeValue={(newVal) => {/* sync central */}}
                      // ...
                 />
             </div>
        )
    }}
</For>
```

## 4. Benefícios e Adequação às Guidelines

1.  **Open-Closed Principle Garantido**: Para adicionar um campo numérico complexo com slider no futuro, basta criar `<SliderCriterionField>` e registrar no dicionário de tipos. Nenhum arquivo central precisa ser modificado substancialmente mais.
2.  **Naming Convention**: Como dita a regra de `frontend-solid.md`, nomes literais e diretos para controle das variáveis (`criterion`, `fieldDefinition` em vez de siglas curtas como `v` ou `c`).
3.  **Encapsulamento Reactivo de Conversões**: As conversões como de/para Data IsoString ficarão responsabilidade estrita do `DateCriterionField` validando localmente antes de despachar via evento `onChangeValue`, desafogando o gigantesco arquivo `useAdvancedSearch.ts` de manter o parsing visual de todos os elementos conhecidos do sistema simultaneamente.

## 5. Próximos Passos de Implementação

1.  Criar a pasta `fields/` dentro de `features/search/` (Colocation Rule).
2.  Implementar a interface de contrato `CriterionFieldRendererProps`.
3.  Desmembrar as lógicas visuais dos `<Show>` legados do `CriteriaBuilder` portando-os para Módulos de Componentes Puros (ex: `DateCriterionField`, `SizeCriterionField`).
4.  Inserir o dicionário de roteamento de componentes.
5.  Atualizar o `CriteriaBuilder` para utilizar a renderização referenciada no Dicionário.
6.  Aplicar os mesmos componentes encapsulados nativamente no modo de edição (dentro do iterador do `QueryEditor.tsx`).
7. Limpar o estado não-necessário de validação visual acumulada no `useAdvancedSearch.ts`.

## 6. Realização (Completed 2026-02-24)

- **[x] Passo 1:** Pasta `fields/` criada dentro de `features/search/` (Colocation Rule).
- **[x] Passo 2:** Assinatura base definida em `CriterionFieldRendererProps`.
- **[x] Passo 3:** Oito (8) componentes dedicados implementados (`DateCriterionField`, `SizeCriterionField`, `TagsCriterionField`, etc).
- **[x] Passo 4:** Dicionário de Roteamento Extensivo implementado como `criterionFieldRegistry`.
- **[x] Passo 5:** `CriteriaBuilder.tsx` profundamente refatorado para utilizar renderização dinâmica do Type Registration, resolvendo complexidade logica (OCP Resolvido).
- **[x] Passo 6:** `QueryEditor.tsx` profundamente refatorado utilizando a exata mesma implementação Component Registry no iterador dos critérios em modo de edição e exibição.
- **[x] Passo 7:** O componente Solid `<Dynamic>` foi otimizado para a instanciação de escopo inline JSX validada pela tipagem estrita do Solid+TypeScript.

- **[x] Passo 8:** Limpeza das armadilhas da reatividade em edição e refatoração estrita da interface nativa `CriterionFieldRendererProps` removendo as atribuições arbitrárias `any` que poluíam a inferência.
- **[x] Passo 9:** Refatoramento dos `displayValue` garantindo fallbacks consistentes para visualização exata de "Ratings (Stars)" e "Formatos Extensão (Name)".

Os resultados foram verificados através de auditoria do linter (`npm run lint`) e checagem forte de tipagem Typescript (`tsc --noEmit`), e ambas verificações apontaram `0 Errors` garantindo a coesão arquitetural desejada.
