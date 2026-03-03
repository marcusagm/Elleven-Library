import { Component, createSignal, createMemo, untrack } from 'solid-js';
import { ColorInput, Slider } from '../../../ui';
import { CriterionFieldRendererProperties, SearchFieldHandler } from './types';

/** Minimum ΔE for exact match (just-noticeable difference threshold). */
const DELTA_E_EXACT = 2.3;

/** Maximum ΔE for broad color family matching. */
const DELTA_E_BROAD = 50;

/**
 * Maps a slider percentage (0-100) to a ΔE threshold value.
 * 0% → exact match (ΔE 2.3), 100% → broad family (ΔE 50).
 */
function sliderPercentageToDeltaE(percentage: number): number {
    return DELTA_E_EXACT + (percentage / 100) * (DELTA_E_BROAD - DELTA_E_EXACT);
}

/**
 * Reverses a ΔE threshold back to a slider percentage.
 * Used when restoring a saved criterion for editing.
 */
function deltaEToSliderPercentage(deltaE: number): number {
    const percentage = ((deltaE - DELTA_E_EXACT) / (DELTA_E_BROAD - DELTA_E_EXACT)) * 100;
    return Math.max(0, Math.min(100, Math.round(percentage)));
}

/**
 * Returns a human-readable label for the current slider position.
 */
function getAccuracyLabel(percentage: number): string {
    if (percentage <= 15) return 'Exact';
    if (percentage <= 50) return 'Similar';
    return 'Broad';
}

/**
 * Extracts hex and proximity from a parsed object.
 * Handles both internal format (with `proximity`) and processed format (with `threshold`).
 */
function extractFromObject(
    objectValue: Record<string, unknown>
): { hex: string; proximity: number } | null {
    if (typeof objectValue.hex !== 'string') return null;

    const proximity =
        typeof objectValue.proximity === 'number'
            ? objectValue.proximity
            : typeof objectValue.threshold === 'number'
              ? deltaEToSliderPercentage(objectValue.threshold)
              : 50;

    return { hex: objectValue.hex, proximity };
}

/**
 * Extracts hex and proximity from a value that can be either:
 * - JSON string: '{"hex":"#FF0000","proximity":50}' (from internal state)
 * - Processed object: { hex: "#FF0000", threshold: 25 } (from saved criterion)
 */
function parseColorValue(raw: unknown): { hex: string; proximity: number } {
    const fallback = { hex: '#000000', proximity: 50 };

    if (typeof raw === 'string') {
        try {
            const parsed = JSON.parse(raw);
            return extractFromObject(parsed) ?? fallback;
        } catch {
            return raw.startsWith('#') ? { hex: raw, proximity: 50 } : fallback;
        }
    }

    if (raw && typeof raw === 'object') {
        return extractFromObject(raw as Record<string, unknown>) ?? fallback;
    }

    return fallback;
}

/**
 * Search criterion field for color-based asset search.
 * Combines a color picker with a proximity slider controlling the ΔE tolerance.
 */
export const ColorCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    const currentValue = createMemo(() => parseColorValue(properties.value));

    const initialProximity = untrack(() => parseColorValue(properties.value).proximity);
    const [localProximity, setLocalProximity] = createSignal(initialProximity);

    const updateValue = (hex: string, proximity: number) => {
        properties.setValue(JSON.stringify({ hex, proximity }));
    };

    const handleColorChange = (newHex: string) => {
        updateValue(newHex, localProximity());
    };

    const handleProximityChange = (newProximity: number) => {
        setLocalProximity(newProximity);
        updateValue(currentValue().hex, newProximity);
    };

    return (
        <div style={{ display: 'flex', 'flex-direction': 'column', gap: '8px', width: '100%' }}>
            <ColorInput
                size={properties.size || 'md'}
                value={currentValue().hex}
                onChange={handleColorChange}
            />
            <div style={{ display: 'flex', 'align-items': 'center', gap: '8px' }}>
                <Slider
                    value={localProximity()}
                    minimumValue={0}
                    maximumValue={100}
                    stepValue={5}
                    onValueChange={handleProximityChange}
                    showTooltip={true}
                    formatValue={value => `ΔE ${sliderPercentageToDeltaE(value).toFixed(1)}`}
                />
                <span
                    style={{
                        'font-size': 'var(--p-font-size-xxs)',
                        color: 'var(--text-secondary)',
                        'white-space': 'nowrap',
                        'min-width': '40px',
                        'text-align': 'right'
                    }}
                >
                    {getAccuracyLabel(localProximity())}
                </span>
            </div>
        </div>
    );
};

/**
 * Validates the color criterion values.
 */
function validateColorCriterion(value: unknown): Record<string, string> {
    const validationErrors: Record<string, string> = {};
    if (!value) {
        validationErrors.value = 'Color is required';
        return validationErrors;
    }

    const parsed = parseColorValue(value);
    if (!parsed.hex || !/^#[0-9A-Fa-f]{6}$/.test(parsed.hex)) {
        validationErrors.value = 'Invalid hex color';
    }

    return validationErrors;
}

/**
 * Handler for color-type search criteria.
 */
export const colorHandler: SearchFieldHandler = {
    component: ColorCriterionField,

    validate: value => validateColorCriterion(value),

    process: value => {
        const parsed = parseColorValue(value);
        const threshold = sliderPercentageToDeltaE(parsed.proximity);

        return {
            finalValue: {
                hex: parsed.hex,
                threshold: Math.round(threshold * 10) / 10
            }
        };
    },

    formatDisplay: (value: unknown) => {
        const parsed = parseColorValue(value);
        const label = getAccuracyLabel(parsed.proximity);
        return `${parsed.hex} (${label})`;
    }
};
