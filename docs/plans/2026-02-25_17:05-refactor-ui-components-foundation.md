# Plano de Refatoração: Componentes Base de UI (Button, Toggle, Switch, Kbd)

**Data:** 2026-02-25
**Status:** Iniciado
**Objetivo:** Elevar a qualidade técnica, documentação e organização dos componentes base seguindo as diretrizes SOLID e o padrão modular do projeto.

---

## 1. Objetivos Técnicos

- **Modularização:** Mover cada componente para sua própria pasta em `src/components/ui/`.
- **Nomenclatura Descritiva:** Eliminar abreviações como `local`, `others`, `props`, `cn`, etc. (Seguindo a regra: "Cada variável deve descrever exatamente sua responsabilidade").
- **Documentação (TSDoc):** Adicionar comentários completos em inglês com descrições, parâmetros, retornos e exemplos práticos.
- **SOLID & SRP:** Extrair lógica complexa (especialmente em `ToggleGroup`) para hooks ou componentes menores.
- **CSS Colocado:** Mover arquivos de estilo para dentro das pastas dos componentes.

---

## 2. Estrutura Proposta

### Button & ButtonGroup
`src/components/ui/Button/`
- `index.ts` (Exportador)
- `Button.tsx` (Componente Raiz)
- `ButtonGroup.tsx` (Agrupador)
- `types.ts` (Definições de variantes e props)
- `button.css`
- `button-group.css`

### Toggle & ToggleGroup
`src/components/ui/Toggle/`
- `index.ts`
- `Toggle.tsx`
- `ToggleGroup.tsx`
- `ToggleGroupItem.tsx` (Extraído do arquivo principal)
- `ToggleGroupContext.tsx` (Contexto tipado)
- `useToggleGroup.ts` (Hook de consumo interno)
- `types.ts`
- `toggle.css`
- `toggle-group.css`

### Switch
`src/components/ui/Switch/`
- `index.ts`
- `Switch.tsx`
- `types.ts`
- `switch.css`

### Kbd
`src/components/ui/Kbd/`
- `index.ts`
- `Kbd.tsx`
- `kbd.css`

---

## 3. Checklist de Implementação

### Fase 1: Button & ButtonGroup
- [x] Criar pasta e mover arquivos.
- [x] Refatorar `types.ts` com documentação exaustiva.
- [x] Refatorar `Button.tsx`:
    - Renomear `local` para `localProperties`.
    - Renomear `others` para `remainingProperties`.
    - Adicionar TSDoc completo.
- [x] Refatorar `ButtonGroup.tsx` seguindo o mesmo padrão.
- [x] Criar `index.ts`.

### Fase 2: Toggle & ToggleGroup
- [x] Criar pasta e mover arquivos.
- [x] Refatorar `types.ts`.
- [x] Refatorar `Toggle.tsx`.
- [x] Refatorar `ToggleGroup.tsx`:
    - Separar contexto e logicamente modularizar.
- [x] Criar `index.ts`.

### Fase 3: Switch & Kbd
- [x] Criar pastas individuais.
- [x] Refatorar `Switch.tsx`:
    - Aplicar nomenclatura descritiva.
    - Adicionar TSDoc.
- [x] Refatorar `Kbd.tsx`:
    - Aplicar nomenclatura descritiva.
    - Adicionar TSDoc.
- [x] Criar `index.ts` em cada pasta.

### Fase 4: Integração Final
- [x] Atualizar `src/components/ui/index.ts`.
- [x] Verificar importações em arquivos dependentes (Baseado em exports do `index.ts`).
- [x] Validar com Linter e Type Check.
- [x] Documentar no plano final.

---

## 4. Próximos Passos
1. Iniciar a Fase 1 (Button).
