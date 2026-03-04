import { Component, createMemo, JSX, Show } from 'solid-js';
import { ColorInput, Slider } from '../../../ui';
import { CriterionFieldRendererProperties } from './types';

/** Minimum ΔE for exact match (just-noticeable difference threshold). */
const DELTA_E_EXACT = 2.3;

/** Maximum ΔE for broad color family matching. */
const DELTA_E_BROAD = 50;

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
export const ColorCriterionField: Component<CriterionFieldRendererProperties> = (
    componentProperties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Creates a memoized color value from the component's value property.
     * The memoization ensures that the color value is only re-parsed when the value changes.
     *
     * @returns {Memo<{ hex: string; proximity: number }>} The memoized color value.
     */
    const currentValue = createMemo(() => parseColorValue(componentProperties.value));

    /**
     * Checks if the current comparison logic expects an exact match.
     *
     * @returns {boolean} True if the comparison operator is 'exact', false otherwise.
     */
    const isExactMode = () => componentProperties.comparisonOperator === 'exact';

    /**
     * Updates the component's value with a new color and proximity.
     *
     * @param {string} hex - The new color value in hexadecimal format.
     * @param {number} proximity - The new proximity value.
     */
    const updateValue = (hex: string, proximity: number) => {
        componentProperties.setValue(JSON.stringify({ hex, proximity }));
    };

    /**
     * Handles color changes by updating the component's value with the new color and current proximity.
     *
     * @param {string} newHex - The new color value in hexadecimal format.
     */
    const handleColorChange = (newHex: string) => {
        updateValue(newHex, currentValue().proximity);
    };

    /**
     * Handles proximity changes by updating the component's value with the current color and new proximity.
     *
     * @param {number} newProximity - The new proximity value.
     */
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
