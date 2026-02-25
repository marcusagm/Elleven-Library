# Plano de Refatoração: ColorPicker & ColorInput (Arquitetura de Excelência)

Este plano descreve a refatoração completa dos componentes de seleção de cor para atingir o estado de excelência arquitetural, seguindo o padrão de **Compound Components**, os guias de **SOLID** e a integração com o sistema de input do sistema (**Mundam Core Input**).

## 1. Objetivos

- **Decomposição Modular:** Transformar o `ColorPicker` em uma pasta de componentes atômicos.
- **Responsabilidade Única (SRP):** Separar lógica de conversão, gerenciamento de estado e renderização.
- **Qualidade de Código:** Eliminar abreviações, usar nomes descritivos e adicionar TSDoc completo.
- **Acessibilidade & Atalhos:** Integrar navegação por teclado via `src/core/input`.
- **Reutilização:** Permitir que partes do ColorPicker sejam usadas de forma independente se necessário.

---

## 2. Estrutura Proposta

### ColorPicker (`src/components/ui/ColorPicker/`)
- `index.ts`: Ponto de entrada.
- `ColorPicker.tsx`: Root que gerencia o estado e o contexto.
- `ColorArea.tsx`: Controle bidimensional (Saturação e Brilho).
- `ColorSlider.tsx`: Sliders unidimensionais (Matiz/Hue e opcionalmente Alfa).
- `ColorPresets.tsx`: Grades de cores predefinidas.
- `ColorPreview.tsx`: Visualização da cor selecionada.
- `ColorHexInput.tsx`: Campo de entrada hexadecimal interno.
- `useColorPicker.ts`: Hook para gerenciamento de estado e interação (drag/teclado).
- `utils.ts`: Funções puras de conversão (Hex <-> HSB).
- `types.ts`: Definições de interfaces e tipos.
- `color-picker.css`: Estilos específicos do seletor.

### ColorInput (`src/components/ui/ColorInput/`)
- `index.ts`: Ponto de entrada.
- `ColorInput.tsx`: Componente de campo de texto com Popover integrado.
- `ColorSwatch.tsx`: O gatilho (trigger) que mostra a cor atual.
- `types.ts`: Interfaces de props.
- `color-input.css`: Estilos específicos do input de cor.

---

## 3. Etapas de Implementação

### Fase 1: Fundação & Utilitários
1. Criar as pastas e mover os arquivos `.css`.
2. Implementar `utils.ts` com nomes descritivos:
    - `convertHexadecimalToHueSaturationBrightness`
    - `convertHueSaturationBrightnessToHexadecimal`
    - `validateHexadecimalColor`
3. Definir `types.ts` com TSDoc completo.

### Fase 2: Lógica e Estado (Hook)
1. Implementar `useColorPicker.ts`:
    - Gerenciamento de sinais reativos.
    - Lógica de arrasto (drag) desacoplada da UI.
    - Integração com `src/core/input` para navegação por setas (Shift para passos maiores).

### Fase 3: Componentes do ColorPicker
1. Implementar `ColorPicker.tsx` (Provider).
2. Implementar sub-componentes (Area, Slider, Presets, etc.).

### Fase 4: Refatoração do ColorInput
1. Criar novo `ColorInput` utilizando os novos componentes e o `Popover` padrão.
2. Garantir compatibilidade com as props do `Input` base.

### Fase 5: Integração e Limpeza
1. Atualizar as referências em:
    - `FontToolbar.tsx`
    - `ModelToolbar.tsx`
    - `TagContextMenu.tsx`
2. Remover os arquivos antigos:
    - `src/components/ui/ColorPicker.tsx`
    - `src/components/ui/ColorInput.tsx`

---

## 4. Definição de Nomes (Anti-Abreviação)

| Antigo | Novo (Exemplos) |
|--------|-----------------|
| `hsb`  | `hueSaturationBrightness` |
| `sb`   | `saturationBrightness` |
| `hex`  | `hexadecimal` |
| `step` | `movementStep` |
| `ref`  | `elementReference` |
| `btn`  | `button` |
| `val`  | `value` |

---

## 5. Verificação e Testes

- [ ] Verificar se a reatividade do Solid.js é mantida (sem desestruturação de props).
- [ ] Testar navegação por teclado em cada sub-controle.
- [ ] Validar se as cores `transparent` continuam funcionando corretamente.
- [ ] Gerar documentação TSDoc e verificar clareza.
