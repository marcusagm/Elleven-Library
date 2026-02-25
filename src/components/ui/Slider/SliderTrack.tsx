import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';

/**
 * Properties for the SliderTrack component.
 */
interface SliderTrackProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional class name. */
    class?: string;
    /** Track content. */
    children?: JSX.Element;
}

/**
 * The track component for the slider.
 * It handles pointer events for clicking and dragging to update the slider value.
 *
 * @param componentProperties - Properties for the SliderTrack.
 * @returns The rendered track element.
 */
export const SliderTrack: Component<SliderTrackProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, [
        'class',
        'children',
        'onPointerDown'
    ]);
    const slider = useSlider();

    const calculateValueFromPosition = (clientX: number, clientY: number) => {
        if (!slider.trackReference.ref) return slider.value();

        const rect = slider.trackReference.ref.getBoundingClientRect();
        let ratio: number;

        if (slider.orientation() === 'vertical') {
            ratio = 1 - (clientY - rect.top) / rect.height;
        } else {
            ratio = (clientX - rect.left) / rect.width;
        }

        const rawValue =
            slider.minimumValue() + ratio * (slider.maximumValue() - slider.minimumValue());

        // Clamp and round to step
        const clampedValue = Math.min(
            slider.maximumValue(),
            Math.max(slider.minimumValue(), rawValue)
        );
        const steps = Math.round((clampedValue - slider.minimumValue()) / slider.stepValue());
        const roundedValue = slider.minimumValue() + steps * slider.stepValue();

        return Math.min(slider.maximumValue(), Math.max(slider.minimumValue(), roundedValue));
    };

    const handlePointerDown = (event: PointerEvent) => {
        if (slider.isDisabled()) return;

        event.preventDefault();
        slider.setIsDragging(true);

        const newValue = calculateValueFromPosition(event.clientX, event.clientY);
        slider.setValue(newValue);

        const handlePointerMove = (moveEvent: PointerEvent) => {
            const currentNewValue = calculateValueFromPosition(
                moveEvent.clientX,
                moveEvent.clientY
            );
            slider.setValue(currentNewValue);
        };

        const handlePointerUp = () => {
            slider.setIsDragging(false);
            slider.commitValue(slider.value());
            document.removeEventListener('pointermove', handlePointerMove);
            document.removeEventListener('pointerup', handlePointerUp);
        };

        document.addEventListener('pointermove', handlePointerMove);
        document.addEventListener('pointerup', handlePointerUp);

        // Pass event to potential custom handler
        const externalPointerDownHandler = localProperties.onPointerDown;
        if (typeof externalPointerDownHandler === 'function') {
            // Cast to a generic function to satisfy callability, then pass event as any
            // to bridge the gap between native and Solid events.
            (externalPointerDownHandler as (event: unknown) => void)(event);
        } else if (Array.isArray(externalPointerDownHandler)) {
            const [handler, data] = externalPointerDownHandler;
            (handler as (data: unknown, event: unknown) => void)(data, event);
        }
    };

    return (
        <div
            ref={element => {
                slider.trackReference.ref = element;
            }}
            class={cn('ui-slider-track', localProperties.class)}
            onPointerDown={handlePointerDown}
            {...otherProperties}
        >
            {localProperties.children}
        </div>
    );
};
