import { Component, Show } from 'solid-js';
import { type LucideProps, Inbox } from 'lucide-solid';

/**
 * Properties for the EmptyState component.
 */
interface EmptyStateProps {
    /** Main message to display when the table is empty */
    message?: string;
    /** Supporting description providing context or actions for the user */
    description?: string;
    /** Custom icon component to override the default inbox icon */
    icon?: Component<LucideProps>;
}

/**
 * Renders a visually expressive empty state view for the table.
 *
 * Displays a placeholder icon, a prominent message, and a secondary description
 * when the table contains no data. This helps guide the user on why the
 * view is empty and how they might populate it.
 *
 * @param {EmptyStateProps} props - Layout and content configuration properties.
 * @returns {JSX.Element} The rendered empty state placeholder row.
 *
 * @example
 * <EmptyState
 *   message="No Results Found"
 *   description="Try adjusting your search terms."
 * />
 */
export const EmptyState: Component<EmptyStateProps> = props => {
    return (
        <div class="ui-table-grid-row ui-table-empty-state" role="row" aria-rowindex={2}>
            <div class="ui-table-empty-icon">
                <Show when={props.icon} fallback={<Inbox size={48} />}>
                    {IconComponent => {
                        /** Resolve the icon component from the accessor */
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
