import { Component, Show } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { ImageOff } from 'lucide-solid';
import './empty-state.css';

export interface EmptyStateProps {
    title?: string;
    description?: string;
    icon?: Component<{ size?: number }>;
}

/**
 * EmptyState - Displayed when there are no items to show
 */
export const EmptyState: Component<EmptyStateProps> = props => {
    return (
        <div class="empty-state">
            <div class="empty-state-icon">
                <Dynamic component={props.icon || ImageOff} size={48} />
            </div>
            <h3 class="empty-state-title">{props.title || 'No items to display'}</h3>
            <Show when={props.description}>
                <p class="empty-state-description">{props.description}</p>
            </Show>
        </div>
    );
};
