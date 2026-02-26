import { Component } from 'solid-js';
import { X } from 'lucide-solid';
import { TagOption } from './types';

/**
 * Properties for the TagChip component.
 */
interface TagChipProps {
    /** The tag data to display */
    tag: TagOption;
    /** Callback to remove the tag */
    onRemove: (id: string | number) => void;
    /** Whether the tag interaction is disabled */
    disabled?: boolean;
}

/**
 * A visually styled chip representing a selected tag.
 * Allows removal via a close button.
 *
 * @param props - Properties for the TagChip component.
 * @returns The rendered TagChip component.
 */
export const TagChip: Component<TagChipProps> = props => {
    return (
        <span
            class="ui-tag-chip"
            style={
                props.tag.color
                    ? {
                          'background-color': props.tag.color,
                          color: 'white'
                      }
                    : undefined
            }
        >
            <span class="ui-tag-chip-label">{props.tag.label}</span>
            <button
                type="button"
                class="ui-tag-chip-remove"
                onClick={event => {
                    event.stopPropagation();
                    props.onRemove(props.tag.id);
                }}
                aria-label={`Remove ${props.tag.label}`}
                disabled={props.disabled}
            >
                <X size={12} />
            </button>
        </span>
    );
};
