import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';

/**
 * Properties for the SliderTrack component.
 */
interface SliderTrackProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional CSS class for custom styling of the track element. */
    class?: string;
    /** Elements to be rendered within the track, usually SliderRange, SliderTicks, and SliderThumb. */
    children?: JSX.Element;
}

/**
 * The SliderTrack component provides the physical container and interactive area for the slider.
 * It detects pointer interactions (clicks and drags) to calculate and update the slider's value
 * based on the relative position of the pointer along the track.
 *
 * @param componentProperties - Properties for the SliderTrack.
 * @returns The rendered track div element.
 */
export const SliderTrack: Component<SliderTrackProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, [
        'class',
        'children',
        'onPointerDown'
    ]);
    const slider = useSlider();

    /**
     * Calculates the slider's numeric value berdasarkan pointer coordinates relative to the track bounds.
     * Takes into account orientation (horizontal/vertical) and range constraints (min/max/step).
     *
     * @param clientX - The X coordinate of the pointer.
     * @param clientY - The Y coordinate of the pointer.
     * @returns The calculated, clamped, and stepped numeric value.
     */
    const calculateValueFromPosition = (clientX: number, clientY: number) => {
        if (!slider.trackReference.ref) return slider.value();

        const trackBoundingRect = slider.trackReference.ref.getBoundingClientRect();
        let positionRatio: number;

        if (slider.orientation() === 'vertical') {
            // In vertical mode, the bottom of the track is usually 0% and the top is 100%.
            positionRatio = 1 - (clientY - trackBoundingRect.top) / trackBoundingRect.height;
        } else {
            // In horizontal mode, the left of the track is usually 0% and the right is 100%.
            positionRatio = (clientX - trackBoundingRect.left) / trackBoundingRect.width;
        }

        const rawCalculatedValue =
            slider.minimumValue() + positionRatio * (slider.maximumValue() - slider.minimumValue());

        // Clamp the raw value to ensure it stays within the defined range bounds.
        const clampedValue = Math.min(
            slider.maximumValue(),
            Math.max(slider.minimumValue(), rawCalculatedValue)
        );

        // Round the clamped value to the nearest incremental step.
        const numberOfSteps = Math.round(
            (clampedValue - slider.minimumValue()) / slider.stepValue()
        );
        const roundedValue = slider.minimumValue() + numberOfSteps * slider.stepValue();

        // Final clamp to handle potential floating point precision issues at the boundaries.
        return Math.min(slider.maximumValue(), Math.max(slider.minimumValue(), roundedValue));
    };

    /**
     * Handles the pointer down interaction on the track.
     * Initiates dragging state and attaches movement listeners to the document for a fluid experience.
     *
     * @param event - The pointer event from the user interaction.
     */
    const handlePointerDown = (event: PointerEvent) => {
        if (slider.isDisabled()) return;

        // Prevent default browser behavior like text selection during dragging.
        event.preventDefault();
        slider.setIsDragging(true);

        // Calculate and set initial value on pointer down (allows "clicking" the track to jump).
        const newValue = calculateValueFromPosition(event.clientX, event.clientY);
        slider.setValue(newValue);

        /**
         * Actively updates the value as the pointer moves across the document.
         */
        const handlePointerMove = (moveEvent: PointerEvent) => {
            const currentNewValue = calculateValueFromPosition(
                moveEvent.clientX,
                moveEvent.clientY
            );
            slider.setValue(currentNewValue);
        };

        /**
         * Cleans up listeners and finalize state when the pointer is released anywhere.
         */
        const handlePointerUp = () => {
            slider.setIsDragging(false);
            slider.commitValue(slider.value());
            document.removeEventListener('pointermove', handlePointerMove);
            document.removeEventListener('pointerup', handlePointerUp);
        };

        // Attach listeners to document to ensure dragging works even if the pointer leaves the track area.
        document.addEventListener('pointermove', handlePointerMove);
        document.addEventListener('pointerup', handlePointerUp);

        // Notify any external listeners attached to the track about the pointer interaction.
        const externalPointerDownHandler = localProperties.onPointerDown;
        if (typeof externalPointerDownHandler === 'function') {
            // Double cast to bridge the gap between native events and potentially custom handler types.
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
