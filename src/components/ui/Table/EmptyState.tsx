import { Component, Show } from 'solid-js';
import { type LucideProps, Inbox } from 'lucide-solid';

interface EmptyStateProps {
    message?: string;
    description?: string;
    icon?: Component<LucideProps>;
}

/**
 * Renders the empty state view for the table.
 * Displays a placeholder icon and message when no data is available.
 *
 * @param {EmptyStateProps} props - Layout and content configuration.
 */
export const EmptyState: Component<EmptyStateProps> = props => {
    return (
        <div class="ui-table-grid-row ui-table-empty-state" role="row" aria-rowindex={2}>
            <div class="ui-table-empty-icon">
                <Show when={props.icon} fallback={<Inbox size={48} />}>
                    {IconComponent => {
                        const CustomIcon = IconComponent();
                        return <CustomIcon size={48} />;
                    }}
                </Show>
            </div>
            <div class="ui-table-empty-message">{props.message || 'No items to display'}</div>
            <div class="ui-table-empty-description">
                {props.description || 'Adjust your filters or add items to see data here.'}
            </div>
        </div>
    );
};
