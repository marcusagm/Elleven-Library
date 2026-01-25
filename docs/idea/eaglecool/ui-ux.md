# Interface do Eagle.cool

## 1. Arquitetura de Layout: O Modelo de Três Painéis

A interface do Eagle segue o padrão clássico de softwares de produtividade profissional (como IDEs ou editores de vídeo), dividindo-se em três zonas principais que minimizam a carga cognitiva ao manter ferramentas contextuais sempre à mão.

* **Painel Esquerdo (Navegação e Bibliotecas):**
* Fundo em tom mais escuro para criar separação visual.
* Organização hierárquica de pastas, pastas inteligentes e filtros rápidos (Tags, Cores, Formatos).
* Uso de ícones minimalistas e cores customizáveis para identificação rápida de categorias.


* **Painel Central (Área de Trabalho/Grid):**
* Onde os ativos são exibidos. Utiliza um sistema de **Justified Grid** (tipo Pinterest), que mantém a proporção original das imagens sem cortá-las, facilitando a varredura visual.
* Scroll fluido e carregamento sob demanda (Lazy Loading) para lidar com milhares de itens sem engasgos.


* **Painel Direito (Inspetor de Metadados):**
* Exibe detalhes do arquivo selecionado. É contextual: se nada for selecionado, mostra estatísticas da pasta atual.
* Inclui a paleta de cores extraída, notas, tags e dados técnicos.





## 2. Design Visual e Estética (Look & Feel)

O Eagle utiliza uma estética **Dark Mode por padrão** (embora ofereça temas claros), o que reduz a fadiga ocular durante longas sessões de curadoria e faz com que as cores dos ativos importados "saltem" na tela.

* **Minimalismo Funcional:** Bordas finas, ausência de sombras pesadas e uso de espaços negativos garantem que a interface não compita com as referências visuais do usuário.
* **Feedback Visual:** Estados de *hover* (passar o mouse) são sutis, mas informativos. Miniaturas de vídeo iniciam o *scrubbing* (visualização rápida) instantaneamente, fornecendo uma resposta tátil ao movimento do mouse.
* **Tipografia:** Utiliza fontes de sistema (San Francisco no macOS, Segoe UI no Windows) para garantir legibilidade e uma sensação de "app nativo", integrada ao sistema operacional.



## 3. Padrões de Interação e Affordance (UX)

A experiência do usuário no Eagle é moldada para **velocidade**. Quase todas as ações principais podem ser executadas via teclado ou gestos simples.

### Gestos e Interações Diretas

* **Drag & Drop Ubíquo:** Você pode arrastar arquivos de qualquer lugar para o Eagle, e do Eagle para qualquer software criativo. A interface responde visualmente destacando as áreas de soltura.
* **Quick Search (Ctrl/Cmd + J):** Uma barra de busca "fuzzy" centralizada (estilo Spotlight ou Raycast) que permite saltar para qualquer pasta ou tag sem usar o mouse.

### Microinterações Inteligentes

* **Rating de 1 a 5:** Atalhos numéricos rápidos permitem classificar ativos enquanto o usuário navega com as setas do teclado.
* **Seleção em Lote:** A interação de "clicar e arrastar" para selecionar múltiplos itens é precisa, permitindo a aplicação de tags em massa com um simples arrastar para o painel lateral.



## 4. Diferenciais de UX para Criativos

O Eagle resolve "dores" específicas do público de design que gerenciadores de arquivos comuns (Finder/Explorer) ignoram:

1. **Visualização de Cores como Interação:** A capacidade de clicar em uma cor no inspetor e ver todos os outros arquivos da biblioteca que compartilham aquele tom é uma UX de descoberta poderosa.
2. **Anotações Contextuais:** Em vez de notas gerais, o usuário pode clicar em uma área específica da imagem para comentar. Visualmente, isso é indicado por marcadores numerados que aparecem apenas quando o inspetor está aberto, mantendo a imagem limpa.
3. **Filtragem por Proporção:** Um seletor visual permite filtrar imagens por "Retrato", "Paisagem" ou "Quadrado", uma necessidade funcional crítica para UX designers que buscam referências para dispositivos específicos.



> **Nota do Analista:** A UI do Eagle é um estudo de caso sobre como equilibrar **poder** e **simplicidade**. Ela não tenta ser "bonita" através de ornamentos, mas sim através da ordem e da funcionalidade, o que gera uma percepção de alta confiabilidade no usuário profissional.

Como um arquiteto de software focado em eficiência operacional, estruturei este guia para transformar a curva de aprendizado em um fluxo de trabalho de alta performance. O segredo da UX do **Eagle** não está apenas no que você vê, mas na velocidade com que você pode manipular o que vê.



# Guia de atalhos e fluxos de trabalho otimizados

## 1. O Kit de Sobrevivência do Power User (Atalhos Críticos)

Para dominar a interface sem tirar as mãos do teclado, estes são os comandos que definem a fluidez da experiência:

| Ação | Atalho (Win/Mac) | Impacto na UX |
|  |  |  |
| **Busca Rápida / Pular para** | `Ctrl/Cmd + J` | O "coração" da navegação. Salta para qualquer pasta ou tag instantaneamente. |
| **Classificação (Rating)** | `1, 2, 3, 4, 5` | Aplica estrelas ao ativo selecionado sem menus. `0` remove. |
| **Adicionar Tags** | `T` | Abre o pop-over de tags focado no campo de digitação. |
| **Preview Rápido** | `Espaço` | Abre/Fecha o modo de inspeção ampliado (estilo Quick Look do macOS). |
| **Filtro de Cores** | `C` | Foca o seletor de cores para filtrar a visualização atual. |
| **Renomear em Lote** | `Ctrl/Cmd + R` | Dispara o motor de renomeação sequencial para múltiplos itens. |
| **Copiar Link de Origem** | `Ctrl/Cmd + Shift + C` | Copia a URL de onde a imagem foi capturada para o clipboard. |



## 2. Fluxos de Trabalho Otimizados

Abaixo, apresento três fluxos baseados na análise funcional para diferentes necessidades criativas.

### A. O Fluxo da "Curadoria Relâmpago"

Ideal para quando você captura 100+ referências em uma sessão de pesquisa intensa e precisa organizá-las.

1. **Captura:** Use a extensão do browser com `Alt + Clique Direito` para salvar rapidamente sem sair da página.
2. **Triagem:** No Eagle, use as setas do teclado para navegar e os números `1 a 5` para dar rating. Ativos com `1 estrela` serão deletados depois.
3. **Agrupamento:** Pressione `Ctrl/Cmd + A` (selecionar tudo) e use o atalho `T` para aplicar uma tag comum a todos (ex: "Projeto_Alpha").
4. **Refinamento:** Use o filtro lateral para isolar apenas arquivos com `5 estrelas` e mova-os para a pasta final do projeto.

### B. O Fluxo "Design-to-Production"

Focado na integração entre sua biblioteca de referências e o software de design (Figma/Photoshop).

1. **Descoberta:** Pressione `Ctrl/Cmd + J` e digite o nome da pasta de componentes.
2. **Inspeção:** Use `Espaço` para ver o ativo em tamanho real e verifique a paleta de cores no painel direito.
3. **Transferência:** Arraste o arquivo diretamente da grade do Eagle para dentro do seu canvas no Figma.
4. **Consistência:** Clique no código HEX de uma cor no painel do Eagle (ele é copiado automaticamente) e cole no seletor de cores do seu software de design.

### C. O Fluxo de "Higienização de Biblioteca"

Para manter o sistema leve e evitar redundâncias.

1. **Limpeza:** Vá em `Filter > Others > Duplicates` para encontrar arquivos idênticos.
2. **Substituição:** Use a ferramenta de análise de duplicatas para decidir qual versão manter baseada na resolução/tamanho.
3. **Smart Sorting:** Crie uma **Smart Folder** com a regra: `Tags - is empty`. Isso listará todos os arquivos "órfãos" que precisam de classificação.



## 3. Estratégia Visual (Checklist de Inspeção de UI)

Ao analisar as telas do Eagle, você notará padrões de UX que garantem a escalabilidade:

* **Densidade de Informação Variável:** No canto inferior direito, existe um slider de zoom. Ele permite mudar de uma visão macro (muitas miniaturas pequenas) para uma visão focada (detalhes grandes), ajustando a UX ao seu nível de cansaço visual.
* **Foco Contextual:** Quando você seleciona múltiplos arquivos, o painel direito muda para "Batch Processing Mode", mostrando apenas o que é comum entre eles. Isso evita confusão visual.
* **Acessibilidade Cromática:** O seletor de cores não é apenas um círculo cromático; ele permite filtrar por "Cores de UI" comuns, facilitando a vida de quem trabalha com sistemas de cores específicos.



> **Dica Pro:** No menu de configurações, você pode habilitar o "Mouse Gesture" para salvar imagens. É a forma mais rápida de captura: basta segurar o botão direito e arrastar a imagem ligeiramente para o lado.

Esta análise conclui o mapeamento técnico e funcional do **Eagle.cool**. Você possui agora uma visão completa de como a ferramenta opera, sua arquitetura de interface e como extrair o máximo de performance dela.