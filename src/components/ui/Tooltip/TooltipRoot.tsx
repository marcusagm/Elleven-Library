/**
 * Tooltip Root
 *
 * Orchestrates the visibility, triggers, and positioning of a tooltip.
 * Manages timing delays and links compound parts via Context.
 */

import { Component, untrack, onCleanup } from 'solid-js';
import { createId, createControllableSignal, createFloating } from '../../../lib/primitives';
import { TooltipRootProperties, TooltipContextState } from './types';
import { TooltipContext } from './TooltipContext';

/**
 * Controller component for the Tooltip system.
 *
 * @param {TooltipRootProperties} rootProperties - Configuration and children.
 * @returns {JSX.Element} Context provider for compound parts.
 */
export const TooltipRoot: Component<TooltipRootProperties> = rootProperties => {
    /** Reactive signal for visibility. */
    const isVisibleSignal = createControllableSignal({
        value: () => rootProperties.isVisible,
        defaultValue: untrack(() => rootProperties.isDefaultVisible ?? false),
        onChange: (visible: boolean) => rootProperties.onVisibleChange?.(visible)
    });

    const isVisible = () => isVisibleSignal.value();
    const setIsVisible = (visible: boolean) => isVisibleSignal.setValue(visible);

    /** Unique identifier for the tooltip content. */
    const contentId = createId('tooltip');

    /** Delay timer for showing the tooltip. */
    let showDelayTimer: ReturnType<typeof setTimeout> | undefined;

    /** Function to show with delay. */
    const show = () => {
        if (rootProperties.isDisabled) {
            return;
        }

        const delay = rootProperties.showDelay ?? 200;
        if (delay === 0) {
            setIsVisible(true);
        } else {
            showDelayTimer = setTimeout(() => setIsVisible(true), delay);
        }
    };

    /** Function to hide instantly and clear timer. */
    const hide = () => {
        if (showDelayTimer) {
            clearTimeout(showDelayTimer);
            showDelayTimer = undefined;
        }
        setIsVisible(false);
    };

    /** Positioning logic. */
    const {
        setTriggerReference,
        setFloatingElement,
        triggerReference,
        floatingElement,
        coordinates
    } = createFloating({
        placement: () => rootProperties.placement ?? 'top',
        offsetValue: () => rootProperties.offsetValue ?? 8,
        isAutoUpdateEnabled: () => isVisible()
    });

    /** Shared context object. */
    const context: TooltipContextState = {
        isVisible,
        show,
        hide,
        contentId,
        placement: () => rootProperties.placement ?? 'top',
        setTriggerReference,
        triggerReference,
        setFloatingElement,
        floatingElement,
        coordinates
    };

    onCleanup(hide);

    return (
        <TooltipContext.Provider value={context}>{rootProperties.children}</TooltipContext.Provider>
    );
};
