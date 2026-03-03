import { Component, createSignal, createMemo } from 'solid-js';
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
 * Returns a human-readable label for the current slider position.
 */
function getAccuracyLabel(percentage: number): string {
    if (percentage <= 15) return 'Exact';
    if (percentage <= 50) return 'Similar';
    return 'Broad';
}

/**
 * Search criterion field for color-based asset search.
 * Combines a color picker with a proximity slider controlling the ΔE tolerance.
 */
export const ColorCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    const currentValue = createMemo(() => {
        const raw = properties.value;
        if (typeof raw === 'string') {
            try {
                return JSON.parse(raw) as { hex: string; proximity: number };
            } catch {
                return { hex: raw || '#000000', proximity: 50 };
            }
        }
        return { hex: '#000000', proximity: 50 };
    });

    const extractInitialProximity = (): number => {
        const raw = properties.value;
        if (typeof raw === 'string') {
            try {
                const parsed = JSON.parse(raw);
                return parsed.proximity ?? 50;
            } catch {
                return 50;
            }
        }
        if (raw && typeof raw === 'object' && 'proximity' in raw) {
            return (raw as { proximity: number }).proximity;
        }
        return 50;
    };

    const [localProximity, setLocalProximity] = createSignal(extractInitialProximity());

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

    let parsed: { hex?: string; proximity?: number };
    if (typeof value === 'string') {
        try {
            parsed = JSON.parse(value);
        } catch {
            validationErrors.value = 'Invalid color value';
            return validationErrors;
        }
    } else {
        parsed = value as { hex?: string; proximity?: number };
    }

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
        if (typeof value !== 'string') {
            return { finalValue: JSON.stringify({ hex: '#000000', threshold: 25 }) };
        }

        let parsed: { hex: string; proximity: number };
        try {
            parsed = JSON.parse(value);
        } catch {
            return { finalValue: JSON.stringify({ hex: '#000000', threshold: 25 }) };
        }

        const threshold = sliderPercentageToDeltaE(parsed.proximity ?? 50);

        return {
            finalValue: JSON.stringify({
                hex: parsed.hex,
                threshold: Math.round(threshold * 10) / 10
            })
        };
    },

    formatDisplay: (value: unknown) => {
        try {
            const parsed = typeof value === 'string' ? JSON.parse(value) : value;
            const label = getAccuracyLabel(parsed.proximity ?? 50);
            return `${parsed.hex} (${label})`;
        } catch {
            return String(value);
        }
    }
};
