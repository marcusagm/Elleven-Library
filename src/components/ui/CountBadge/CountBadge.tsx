import { Component, Show, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { Tooltip } from '../Tooltip';
import { CountBadgeProperties } from './types';
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
    const [localProperties, remainingProperties] = splitProps(properties, [
        'count',
        'variant',
        'max',
        'showZero',
        'class'
    ]);

    const activeVariant = () => localProperties.variant || 'secondary';
    const maximumValue = () => localProperties.max ?? 9999;
    const shouldShowZero = () => localProperties.showZero ?? false;

    const shouldShowBadge = () => localProperties.count > 0 || shouldShowZero();

    /**
     * Formats the numeric count into a shorter string representation.
     */
    const formatCountValue = (numericCount: number): string => {
        if (numericCount > maximumValue()) {
            return `${formatCountValue(maximumValue())}+`;
        }
        if (numericCount >= 1000000) {
            return (numericCount / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
        }
        if (numericCount >= 1000) {
            return (numericCount / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
        }
        return numericCount.toString();
    };

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
            {formatCountValue(localProperties.count)}
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
