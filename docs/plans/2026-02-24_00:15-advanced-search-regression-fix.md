# Refinamento da Busca Avançada e Smart Folders - Resolução de Regressões

## Visão Geral

Este documento descreve as etapas, desafios e soluções implementadas para refinar o Modal de Busca Avançada (`AdvancedSearchModal`) após um grande processo de refatoração no sistema de filtros (implementado em `useAdvancedSearch`). A refatoração inicial causou a perda funcional da capacidade de executar buscas temporárias avulsas, travando a interface de buscas apenas no modo estrito de edição de Smart Folders. Adicionalmente, também foram encontrados e solucionados comportamentos incorretos de reatividade nos valores dos formulários durante a reedição dos critérios de busca.

## Contexto e Problemas Encontrados

Após o desacoplamento arquitetural e a criação de hooks composables modulares para o editor de queries (`CriteriaBuilder` e `QueryEditor`), os seguintes problemas colaterais (regressões) emergiram:

1.  **Regressão do Modal ("Sempre em modo Smart Folder"):** O modal foi modificado para exibir estaticamente sempre o título *"Edit Smart Folder"* e exigir a interação *"Save Smart Folder"*, desativando completamente a funcionalidade paralela de submeter buscas livres e temporárias com o botão primário *"Search"*.
2.  **Perda do Valor de Input na Edição:** Quando um critério de busca (ex: Data ou Tamanho de arquivo) era selecionado para alteração, os componentes preenchidos do formulário apareciam vazios ou com tipagens quebrando (`Type Mismatch`), obrigando o usuário a re-inserir todos os dados repetidos nas correções inline.
3.  **Vazamento Estético de Tailwind (Utility Classes):** A barra de salvamento de Smart Folder que ressurgiu com a correção do modal foi injetada com strings arbitrárias inspiradas no modelo Tailwind CSS (`flex items-center gap-2`), quebrando as regras e o Design System nativo configurado via vanilla-CSS do projeto para a UI do App Mundam.

## Passos da Solução Implementada

### 1. Restauração do Rodapé Duplo e Busca Temporária

*   **Identificação:** No arquivo principal engatilhador da interface modal (`SearchToolbar.tsx`), a propriedade paramétrica estava explicitamente preenchida como `isSmartFolderMode={true}` como forma de fallback inseguro da arquitetura pré-refactorização.
*   **Correção:** Modificado o parâmetro para `isSmartFolderMode={!!currentSmartFolder()}` na chamada do `SearchToolbar`. Isso possibilita que:
    *   Sempre que não houver um contexto local atrelado de Smart Folder persistida, a interface abra a Busca Temporária normal;
    *   Exiba a ação primária `Search` e seu conversor interativo `Save as Smart Folder` simultaneamente.
*   **Refinamento de UI Modal:** Retornado a lógica multi-render no `AdvancedSearchModal.tsx` adaptando a exibição do footer do Modal com o botão Desativado (`disabled`) condicional tanto caso o layout de edição requeira um texto nomeando a Smart Folder para gravar ou se os critérios ficarem zerados.

### 2. Aderência Fiel dos Tipos Nativos nos Parâmetros e Formulários

*   **Problema de Conversão de `Date` e `Number`:** O SolidJS (e do Typescript de `useAdvancedSearch`) havia uniformizado qualquer valor serializável da interface em `null | string | number`, e removido a premissa de carregar os valores reais em formato serial na edição local antes de confirmar na montagem principal da hierarquia do estado global e dos componentes de campos visuais avançados customizados (`DateInput` / `NumberInput`).
*   **Ajuste da Tipagem:** O Hook central `useAdvancedSearch.ts` teve a tipagem `SearchValue` embutida para também possuir/entender instâncias puras de `<Date>`.
*   **Tratamento Nativo de Estado Reactivo:** Nos campos subjacentes `CriteriaBuilder.tsx` e `QueryEditor.tsx`, todas as passagens de atributos nas tags `<DateInput>` foram atualizados para recuperar estritamente instâncias via TypeScript Cast (`as Date`).
*   **Garantia na Serialização:** Na interceptação final (`handleConfirmEdit` e `handleAddCriteria`), a inserção dos dados verifica rigidamente via bloco condicional `instanceof Date` se algo precisa ser normalizado via o conversor original `formatToISO`, ou se a variável retém apenas strings, evitando que o campo apague reativamente.
*   **Controle de Indefinição Numérica (`NumberInput`)**: Atualizadas as passagens do `NumberInput` para ler o esvaziamento total do campo e repassar apropriadamente objetos `undefined` através e evitar instabilidade interativa ao usuário (retirando null casting duro anterior).

### 3. Remoção e Saneamento das Classes Estéticas Inadequadas

*   **Limpeza `AdvancedSearchModal.tsx`:** O bloco estrutural para acoplar os botões no Footer obteve suas invocações inline do tipo "Tailwind CSS" arrancadas do código:
    *   Removido `<div class="smart-folder-creator flex items-center gap-2">` substituído por apenas `<div class="smart-folder-creator">`.
    *   Removida injeção opcional prop `wrapperClass="w-48"` no Input trocando por `wrapperClass="smart-folder-input-wrapper"`.
*   **Transferência ao Vanilla CSS:** Em `advanced-search-modal.css`, foi injetada uma seção dedicada que descreve as proporções da estrutura formatada perfeitamente de volta para o padrão limpo do Design System implementado na raiz do projeto.

---

## Próximos Passos e Melhorias Futuras

A refatoração provada aqui alcança sua excelência, mas as futuras interações e aprofundamentos da Arquitetura do Componente de Busca do Mundam podem olhar os seguintes cenários:

-   **Reduzir as Complexidades Ciclimáticas Ciclópicas (ESLint e Code Quality)**
    *   **Obstáculo:** Como o `useAdvancedSearch` consolida vários validadores heterogêneos para `Data`, `Range Numérico` e `Textos Simples` em uma única função "faz-tudo" (Switch/Cases de lógicas longas para `handleAddCriteria` e `handleConfirmEdit`), isso provoca de modo persistente alertas do linter sugerindo redução na Complexidade da Lógica interna local (onde alguns métodos excedem o peso "17", sendo 10 o sugerido aceitável).
    *   **Solução Proposta Futura:** Criar manipuladores separados de parsing subdelegados encapsulados em dicionários de Formatos Baseados no `DataType` (`DateHandler`, `NumberHandler`, etc) que recebem a carga validável, operam nela e repassam um Retorno limpo já com propriedades finalizadas para serem gravadas na lista `Criteria`.
-   **Animações de Expansão/Fechamento dos Campos Extras**
    *   Possívelmente explorar a funcionalidade visual dos Inputs (`Between`) que surgem (com o seu texto "to") através de renderizações transicionais na alteração da Match Option dos campos para agregar ao caráter "Premium".
