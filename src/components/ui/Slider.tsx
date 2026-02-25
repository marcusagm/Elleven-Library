import { Component, JSX, splitProps, createMemo, createSignal, For } from 'solid-js';
import { cn } from '../../lib/utils';
import { createControllableSignal } from '../../lib/primitives';
import { createId } from '../../lib/primitives/createId';
import './slider.css';

export interface SliderProps extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'onChange'> {
    value?: number;
    defaultValue?: number;
    onValueChange?: (value: number) => void;
    onValueCommit?: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    orientation?: 'horizontal' | 'vertical';
    showTooltip?: boolean;
    showTicks?: boolean;
    formatValue?: (value: number) => string;
}

/**
 * Calculates tick values for the slider based on range and step.
 */
const calculateTicks = (minValue: number, maxValue: number, stepValue: number | undefined) => {
    if (stepValue === undefined) return [];

    const range = maxValue - minValue;
    if (range <= 0 || stepValue <= 0) return [];

    const tickCount = Math.floor(range / stepValue);
    if (tickCount > 50) return []; // Avoid too many ticks

    const tickValues = [];
    // Start from 1 to skip min
    // Go up to count, but exclude if it equals max
    for (let index = 1; index <= tickCount; index++) {
        const value = minValue + index * stepValue;
        if (value < maxValue) {
            tickValues.push(value);
        }
    }
    return tickValues;
};

/**
 * Slider component for selecting a value from a range.
 *
 * @example
 * <Slider defaultValue={50} min={0} max={100} />
 *
 * const [volume, setVolume] = createSignal(75);
 * <Slider
 *   value={volume()}
 *   onValueChange={setVolume}
 *   showTooltip
 * />
 */
export const Slider: Component<SliderProps> = props => {
    const [local, others] = splitProps(props, [
        'class',
        'value',
        'defaultValue',
        'onValueChange',
        'onValueCommit',
        'min',
        'max',
        'step',
        'disabled',
        'orientation',
        'showTooltip',
        'showTicks',
        'formatValue',
        'id'
    ]);

    const id = createMemo(() => local.id || createId('slider'));
    const min = () => local.min ?? 0;
    const max = () => local.max ?? 100;
    const step = () => local.step ?? 1;
    const showTicks = () => local.showTicks ?? true;
    const isVertical = () => local.orientation === 'vertical';

    let trackRef: HTMLDivElement | undefined;
    const [isDragging, setIsDragging] = createSignal(false);

    const { value, setValue } = createControllableSignal({
        value: () => local.value,
        defaultValue: local.defaultValue ?? min(),
        onChange: (value: number) => local.onValueChange?.(value)
    });

    const percentage = createMemo(() => {
        const range = max() - min();
        if (range === 0) return 0;
        return ((value() - min()) / range) * 100;
    });

    const ticks = createMemo(() => calculateTicks(min(), max(), local.step));

    const formatValue = (currentValue: number) => {
        return local.formatValue ? local.formatValue(currentValue) : String(currentValue);
    };

    const clamp = (val: number) => {
        return Math.min(max(), Math.max(min(), val));
    };

    const roundToStep = (val: number) => {
        const steps = Math.round((val - min()) / step());
        return min() + steps * step();
    };

    const getValueFromPosition = (clientX: number, clientY: number) => {
        if (!trackRef) return value();

        const rect = trackRef.getBoundingClientRect();
        let ratio: number;

        if (isVertical()) {
            ratio = 1 - (clientY - rect.top) / rect.height;
        } else {
            ratio = (clientX - rect.left) / rect.width;
        }

        const rawValue = min() + ratio * (max() - min());
        return roundToStep(clamp(rawValue));
    };

    const handlePointerDown = (e: PointerEvent) => {
        if (local.disabled) return;

        e.preventDefault();
        setIsDragging(true);

        const newValue = getValueFromPosition(e.clientX, e.clientY);
        setValue(newValue);

        const handlePointerMove = (e: PointerEvent) => {
            const newValue = getValueFromPosition(e.clientX, e.clientY);
            setValue(newValue);
        };

        const handlePointerUp = () => {
            setIsDragging(false);
            local.onValueCommit?.(value());
            document.removeEventListener('pointermove', handlePointerMove);
            document.removeEventListener('pointerup', handlePointerUp);
        };

        document.addEventListener('pointermove', handlePointerMove);
        document.addEventListener('pointerup', handlePointerUp);
    };

    const handleKeyDown = (event: KeyboardEvent) => {
        if (local.disabled) return;

        let newValue = value();
        const bigStep = step() * 10;

        switch (event.key) {
            case 'ArrowRight':
            case 'ArrowUp':
                newValue = clamp(value() + step());
                break;
            case 'ArrowLeft':
            case 'ArrowDown':
                newValue = clamp(value() - step());
                break;
            case 'PageUp':
                newValue = clamp(value() + bigStep);
                break;
            case 'PageDown':
                newValue = clamp(value() - bigStep);
                break;
            case 'Home':
                newValue = min();
                break;
            case 'End':
                newValue = max();
                break;
            default:
                return;
        }

        event.preventDefault();
        setValue(newValue);
        local.onValueCommit?.(newValue);
    };

    return (
        <div
            class={cn(
                'ui-slider',
                `ui-slider-${local.orientation || 'horizontal'}`,
                local.disabled && 'ui-slider-disabled',
                local.class
            )}
            {...others}
        >
            <div ref={trackRef} class="ui-slider-track" onPointerDown={handlePointerDown}>
                <div
                    class="ui-slider-range"
                    style={{
                        [isVertical() ? 'height' : 'width']: `${percentage()}%`
                    }}
                />
                {showTicks() && (
                    <For each={ticks()}>
                        {tickValue => (
                            <div
                                class="ui-slider-tick"
                                style={{
                                    [isVertical() ? 'bottom' : 'left']:
                                        `${((tickValue - min()) / (max() - min())) * 100}%`
                                }}
                            />
                        )}
                    </For>
                )}
                <div
                    class={cn('ui-slider-thumb', isDragging() && 'ui-slider-thumb-active')}
                    style={{
                        [isVertical() ? 'bottom' : 'left']: `${percentage()}%`
                    }}
                    role="slider"
                    tabindex={local.disabled ? -1 : 0}
                    aria-valuemin={min()}
                    aria-valuemax={max()}
                    aria-valuenow={value()}
                    aria-valuetext={formatValue(value())}
                    aria-orientation={local.orientation || 'horizontal'}
                    aria-disabled={local.disabled}
                    onKeyDown={handleKeyDown}
                >
                    {local.showTooltip && (
                        <div class="ui-slider-tooltip">{formatValue(value())}</div>
                    )}
                </div>
            </div>

            {/* Hidden input for form submission */}
            <input
                type="range"
                id={id()}
                min={min()}
                max={max()}
                step={step()}
                value={value()}
                disabled={local.disabled}
                class="ui-slider-input"
                tabindex={-1}
                aria-hidden="true"
            />
        </div>
    );
};
