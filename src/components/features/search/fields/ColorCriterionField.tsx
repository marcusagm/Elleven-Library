import { Component, createMemo, Show } from 'solid-js';
import { ColorInput, Slider } from '../../../ui';
import { CriterionFieldRendererProperties, SearchFieldHandler } from './types';

/** Minimum ΔE for exact match (just-noticeable difference threshold). */
const DELTA_E_EXACT = 2.3;

/** Maximum ΔE for broad color family matching. */
const DELTA_E_BROAD = 50;

/**
 * Maps a tolerance percentage to a ΔE threshold value.
 *
 * @param {number} percentage - The percentage from 0 to 100.
 * @returns {number} The corresponding ΔE value.
 */
function sliderPercentageToDeltaE(percentage: number): number {
    return DELTA_E_EXACT + (percentage / 100) * (DELTA_E_BROAD - DELTA_E_EXACT);
}

/**
 * Reverses a ΔE threshold back to a slider percentage.
 *
 * @param {number} deltaE - The CIEDE76 ΔE threshold.
 * @returns {number} The percentage from 0 to 100.
 */
function deltaEToSliderPercentage(deltaE: number): number {
    const percentage = ((deltaE - DELTA_E_EXACT) / (DELTA_E_BROAD - DELTA_E_EXACT)) * 100;
    return Math.max(0, Math.min(100, Math.round(percentage)));
}

/**
 * Returns a human-readable label for the current match tolerance.
 *
 * @param {number} percentage - The percentage accuracy.
 * @returns {string} The text label for the given percentage.
 */
function getMatchLabel(percentage: number): string {
    if (percentage === 0) return 'Exact';
    if (percentage <= 25) return 'Very Similar';
    if (percentage <= 50) return 'Similar';
    if (percentage <= 75) return 'Related';
    return 'Broad';
}

/**
 * Extracts hexadecimal color and proximity slider values from an internal layout object.
 *
 * @param {Record<string, unknown>} objectValue - The unknown object reference.
 * @returns {{ hex: string; proximity: number } | null} The extracted color value or null.
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
 * Resolves unknown state contents back into a safe internal color state format.
 *
 * @param {unknown} rawValue - The unverified internal value.
 * @returns {{ hex: string; proximity: number }} The unified formatting.
 */
function parseColorValue(rawValue: unknown): { hex: string; proximity: number } {
    const fallback = { hex: '#000000', proximity: 50 };

    if (typeof rawValue === 'string') {
        try {
            const parsed = JSON.parse(rawValue);
            return extractFromObject(parsed) ?? fallback;
        } catch {
            return rawValue.startsWith('#') ? { hex: rawValue, proximity: 50 } : fallback;
        }
    }

    if (rawValue && typeof rawValue === 'object') {
        return extractFromObject(rawValue as Record<string, unknown>) ?? fallback;
    }

    return fallback;
}

/**
 * Renders an input group for color-based search criteria using a color picker and tolerance slider.
 *
 * @param {CriterionFieldRendererProperties} componentProperties - Component properties containing state and callback handlers.
 * @returns {JSX.Element} The rendered component.
 */
export const ColorCriterionField: Component<
    CriterionFieldRendererProperties
> = componentProperties => {
    // Defines current value as derived state completely controlled by the parent component.
    const currentValue = createMemo(() => parseColorValue(componentProperties.value));
    const isExactMode = () => componentProperties.comparisonOperator === 'exact';

    const updateValue = (hex: string, proximity: number) => {
        componentProperties.setValue(JSON.stringify({ hex, proximity }));
    };

    const handleColorChange = (newHex: string) => {
        updateValue(newHex, currentValue().proximity);
    };

    const handleProximityChange = (newProximity: number) => {
        updateValue(currentValue().hex, newProximity);
    };

    return (
        <div class="color-input-group">
            <div class="color-input-wrapper">
                <ColorInput
                    size={componentProperties.size || 'md'}
                    value={currentValue().hex}
                    onChange={handleColorChange}
                />
            </div>

            <Show when={!isExactMode()}>
                <div class="tolerance-slider-group">
                    <span class="tolerance-slider-label">Tolerance</span>
                    <div class="tolerance-slider-wrapper">
                        <Slider
                            value={currentValue().proximity}
                            minimumValue={0}
                            maximumValue={100}
                            showTicks={false}
                            onValueChange={handleProximityChange}
                            showTooltip={true}
                        />
                    </div>
                    <span class="tolerance-match-label">
                        {getMatchLabel(currentValue().proximity)}
                    </span>
                </div>
            </Show>
        </div>
    );
};

/**
 * Validates the color criterion values.
 *
 * @param {unknown} fieldValue - The field's raw JSON value.
 * @returns {Record<string, string>} A dictionary of potential errors.
 */
function validateColorCriterion(fieldValue: unknown): Record<string, string> {
    const validationErrors: Record<string, string> = {};
    if (!fieldValue) {
        validationErrors.value = 'Color is required';
        return validationErrors;
    }

    const parsed = parseColorValue(fieldValue);
    if (!parsed.hex || !/^#[0-9A-Fa-f]{6}$/.test(parsed.hex)) {
        validationErrors.value = 'Invalid hex color';
    }

    return validationErrors;
}

/**
 * Handler implementation for color-type search criteria.
 */
export const colorHandler: SearchFieldHandler = {
    component: ColorCriterionField,

    validate: value => validateColorCriterion(value),

    process: (value, _value2, operator) => {
        const parsed = parseColorValue(value);
        const threshold =
            operator === 'exact' ? DELTA_E_EXACT : sliderPercentageToDeltaE(parsed.proximity);

        return {
            finalValue: {
                hex: parsed.hex,
                threshold: Math.round(threshold * 10) / 10
            }
        };
    },

    formatDisplay: (value, _value2, operator) => {
        const parsed = parseColorValue(value);
        if (operator === 'exact') {
            return `${parsed.hex} (Exact)`;
        }
        const label = getMatchLabel(parsed.proximity);
        return `${parsed.hex} (Tolerance: ${parsed.proximity}% - ${label})`;
    }
};
