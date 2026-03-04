# Refatoração: Correção Arquitetural do Advanced Search Handlers

## Contexto e Objetivo
Durante o desenvolvimento do componente de busca avançada (*Advanced Search Component*), identificou-se que a lógica de negócios atrelada a cada critério de busca (métodos `validate`, `process` e `formatDisplay`) vinha provocando duas patologias arquiteturais e ferindo as _guidelines_ do projeto:

1. **Duplicação de Código (Front-end e Core):** Inúmeras lógicas e verificações coabitavam de forma repetida tanto nas interfaces visuais de Reactivity do SolidJS (ex: `ColorCriterionField.tsx`) quanto nos processadores base do estado (Core).
2. **Padrão "God-File":** A centralização de toda a lógica do núcleo num único arquivo genérico em `src/core/store/filter/logic/handlers.ts` violava flagrantemente as regras de estabilidade da arquitetura (`core-architecture.md`), correndo risco de extrapolar indefinidamente seu tamanho (alcançando o antipadrão _God File_) assim que a demanda de campos crescesse.

A finalidade fundamental desta etapa foi adotar e estabelecer o **Core State (Store) como Única Fonte da Verdade** num formato *Strategy*, reestruturando e isolando modularmente todas as responsabilidades.

## Passo a Passo da Implementação

### 1. Higienização dos Módulos Residenciais Puros (Solid.js)
Toda e qualquer lógica validatória ou de processamento em massa do cliente foi severamente extirpada dos componentes visuais da busca, isolando tais elementos como Componentes Puros de UI/DOM render.
- Arquivos profundamente limpos em `src/components/features/search/fields/`: 
  - `ColorCriterionField.tsx`
  - `DateCriterionField.tsx`
  - `FolderCriterionField.tsx`
  - `NumberCriterionField.tsx`
  - `RatingCriterionField.tsx`
  - `SelectCriterionField.tsx`
  - `SizeCriterionField.tsx`
  - `TagsCriterionField.tsx`
  - `TextCriterionField.tsx`
- Refatoração do `index.ts` frontend-registry, que passou a atuar como um _Composer_, absorvendo a interface e mesclando dinamicamente os componentes limpos com a autêntica lógica do Core pelo objeto `criterionHandlerRegistry`.

### 2. Desmembramento do "God-File" (`handlers.ts`)
Para anular o risco latente de engessar o Store com milhares de linhas e resolver o gargalo para testes limpos, o arquivo central superpovoado `logic/handlers.ts` (~300 linhas) foi completamente removido e refatorado em micro-arquivos de **Responsabilidade Única** na nova diretoria `src/core/store/filter/handlers/`.

- **Arquitetura Strategy Aplicada em Arquivos Isolados:**
  - `colorLogic.ts`: Dedicado rigorosamente ao complexo threshold LAB (∆E).
  - `dateLogic.ts`: Focado em escopo temporal ISO.
  - `folderLogic.ts`, `numberLogic.ts`, `ratingLogic.ts`, `selectLogic.ts`, `tagsLogic.ts`, `textLogic.ts`.
  - `sizeLogic.ts`: Encapado por multiplicadores da balança de bytes computacionais.

- **Arquitetura de Exportação e Base:** 
  - `types.ts`: Garantindo de interfaces limpas (`SearchFieldLogic` e `SearchValue`).
  - `utils.ts`: Utilitários reaproveitáveis sem contexto específico (ex: método de verificação de não-vazio/nulos).
  - `index.ts`: Um repositório barril (Barrel File) que readaptou de forma transparente as junções de imports e devolveu `criterionLogicRegistry`. Dessa forma assegurando a conformidade restrita ao conceito de Substiituição de Liskov por parte dos agregadores em `filterActions`.

### 3. Ajuste de Conformidade (Code Smell e Linter)
- Abate das metodologias desnecessárias (_Dead Code_), como a regra `sliderPercentageToDeltaE` obsoletada no frontend.
- Tipagem estrita declarativa nos iteradores dos _High Order Functions_ (ex: `.find()`) devido as exigências nativas de `strict` mode no Solid/TS, garantindo a solidez dos atributos da constante `SIZE_UNITS`.

## Obstáculos Encontrados

- **Barreira de Acoplamento:** Extrair o epicentro relacional da camada Front/View e remanejá-lo de forma limpa como Singleton logic sem corromper as _actions_ dos botões foi desafiador. A saída _Registry composer Adapter_ adotada supriu o impasse graciosamente.
- **Checagem Rígida do TS no Find:** As definições de Array não inferiam corretamente tipos aninhados personalizados em literais genéricos de escopo. As devidas injeções contratuais tiveram que ser passadas explicitly no escopo dos métodos de map do compilador.
- **Lint de Extensões Imports:** As importações locais foram geradas sob alerta do compilador no momento da adição arbitrária da extensão de arquivo `.ts`, exigindo readequação silenciosa dos scripts de translação para imports puros ES sem extensão explícita (`from './utils'`).

## Próximos Passos e Melhorias Futuras

1. **Deep Testing nos Handlers Core:** Visto que `colorLogic.ts` ou `numberLogic.ts` são agora classes literais autônomas sem escopo reativo atrelado ou amarras do DOM, pode (e deve) fluir de maneira muito mais suave uma vindoura bateria inteiramente TDD baseada em testes unitários rígidos com Vitest/Jest (Ex: Validar o calculo DeltaE passando variáveis fictícias para a constante de proximidade limitrofe).
2. **Delegação nativa ao schema Zod:** Avaliar futuramente se a rotina `validate()` nesses handlers não pode ser terceirizada ao processo de refines (`z.string().refine(...)`) já contidos no arquivo irmão `schemas.ts` de tal forma cortando em ~40% a verbosidade validacional de lógicos em favor das bibliotecas unissexuais padronizadas.
3. **Lazy-loading Integrado**: Criar um mecanismo na interface em que as lógicas pesadas só seriam exigidas (import dinâmico) conforme o usuário habilitasse a "pill" correspondente à sua respectiva métrica em uso.
