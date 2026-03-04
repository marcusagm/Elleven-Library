# Brainstorm: Padronização de Funções Utilitárias

## Conclusão de Migração e Refatoração (Option C)

- `src/utils/format.ts` foi expandido para agregar `formatCompactNumber` e `formatTime`. Todos os componentes que duplicavam formatação matemática de interfaces agora dependem dessa única fonte.
- `src/utils/color.ts` foi criado e unificou os tipos como `HueSaturationBrightness` e conversores rigorosos hexadecimais de forma a compartilhar entre o *Inspector* (que necessitava de cartesianos precisos) e o *ColorPicker*.
- Todos os arquivos utilitários locais duplicantes de escopos globais dos componentes (arquivos `utils.ts`) estão sendo deletados.
- Arquivos modificados: `AssetCardOverlay.tsx`, `VideoControls.tsx`, `VideoSeekbar.tsx`, `AudioControls.tsx`, `CountBadge.tsx`, `CommonMetadata.tsx`, `colorClusteringUtils.ts`, `colorHarmonyUtils.ts`, e `ColorPicker`.
