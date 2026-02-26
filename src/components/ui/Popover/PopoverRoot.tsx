/**
 * Popover Root
 *
 * Orchestrator component that manages the open state, accessibility IDs,
 * and positioning logic for the Popover compound component parts.
 */

import { Component, untrack } from 'solid-js';
import { createId, createControllableSignal, createFloating } from '../../../lib/primitives';
import { PopoverRootProperties, PopoverContextState } from './types';
import { PopoverContext } from './PopoverContext';

/**
 * Controller component for the Popover system.
 *
 * @param {PopoverRootProperties} rootProperties - Configuration and children.
 * @returns {JSX.Element} Context provider for compound parts.
 */
export const PopoverRoot: Component<PopoverRootProperties> = rootProperties => {
    /** Reactive signal for the open state. */
    const isOpenSignal = createControllableSignal({
        value: () => rootProperties.isOpen,
        defaultValue: untrack(() => rootProperties.isDefaultOpen ?? false),
        onChange: (open: boolean) => rootProperties.onOpenChange?.(open)
    });

    const isOpen = () => isOpenSignal.value();
    const setIsOpen = (open: boolean) => isOpenSignal.setValue(open);

    /** Unique identifier for the popover content. */
    const contentId = createId('popover');

    /** Positioning logic. */
    const {
        setTriggerReference,
        setFloatingElement,
        triggerReference,
        floatingElement,
        coordinates
    } = createFloating({
        placement: () => rootProperties.placement ?? 'bottom-start',
        offsetValue: () => rootProperties.offsetValue ?? 8,
        isAutoUpdateEnabled: () => isOpen()
    });

    /** Shared context object. */
    const context: PopoverContextState = {
        isOpen,
        setIsOpen,
        contentId,
        setTriggerReference,
        triggerReference,
        setFloatingElement,
        floatingElement,
        coordinates
    };

    return (
        <PopoverContext.Provider value={context}>{rootProperties.children}</PopoverContext.Provider>
    );
};
