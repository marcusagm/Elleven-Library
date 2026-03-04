import { Component, Show, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { Tooltip } from '../Tooltip';
import { CountBadgeProperties } from './types';
import { formatCompactNumber } from '../../../utils/format';
import './count-badge.css';

/**
 * CountBadge component for displaying numeric values with automatic formatting.
 * Displays abbreviated values (e.g., 1.2k, 1M) and shows the exact count in a tooltip.
 *
 * @param {CountBadgeProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered count-badge component.
 *
 * @example
 * <CountBadge count={1234} />
 * // Displays "1.2k" with tooltip "1,234"
 *
 * @example
 * <CountBadge count={5} variant="primary" />
 */
export const CountBadge: Component<CountBadgeProperties> = properties => {
    /**
     * Splits the properties into local properties and remaining properties.
     * @param properties - The properties to split.
     * @returns The split properties.
     */
    const [localProperties, remainingProperties] = splitProps(properties, [
        'count',
        'variant',
        'max',
        'showZero',
        'class'
    ]);

    /**
     * Gets the active variant of the count badge.
     * @returns The active variant of the count badge.
     */
    const activeVariant = () => localProperties.variant || 'secondary';

    /**
     * Gets the maximum value of the count badge.
     * @returns The maximum value of the count badge.
     */
    const maximumValue = () => localProperties.max ?? 9999;

    /**
     * Checks if the count badge should show zero.
     * @returns True if the count badge should show zero, false otherwise.
     */
    const shouldShowZero = () => localProperties.showZero ?? false;

    /**
     * Checks if the count badge should be shown.
     * @returns True if the count badge should be shown, false otherwise.
     */
    const shouldShowBadge = () => localProperties.count > 0 || shouldShowZero();

    /**
     * Renders the count badge.
     * @returns The rendered count badge.
     */
    const RenderedBadge: Component = () => (
        <span
            class={concatenateClasses(
                'ui-count-badge',
                `ui-count-badge-${activeVariant()}`,
                localProperties.class
            )}
            aria-label={`Count: ${localProperties.count.toLocaleString()}`}
            {...remainingProperties}
        >
            {formatCompactNumber(localProperties.count, maximumValue())}
        </span>
    );

    return (
        <Show when={shouldShowBadge()}>
            <Show when={localProperties.count >= 1000} fallback={<RenderedBadge />}>
                <Tooltip
                    content={localProperties.count.toLocaleString()}
                    placement="top"
                    offsetValue={8}
                >
                    <RenderedBadge />
                </Tooltip>
            </Show>
        </Show>
    );
};
