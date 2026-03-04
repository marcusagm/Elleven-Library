import { Component, createSignal, createEffect, untrack } from 'solid-js';
import { Info, FileText, Calendar, HardDrive } from 'lucide-solid';
import { AccordionItem, AccordionHeader, AccordionContent } from '../../../ui';
import { Input } from '../../../ui/Input';
import { StarRating } from './StarRating.tsx';
import { useLibrary } from '../../../../core/hooks';
import { type AssetItem } from '../../../../types';
import { formatFileSize, formatToDisplay } from '../../../../utils/format';
import './CommonMetadata.css';

/**
 * Interface for CommonMetadata component properties.
 */
interface CommonMetadataProps {
    /**
     * The item to display metadata for.
     */
    item: AssetItem | null;
}

/**
 * CommonMetadata component
 *
 * @module CommonMetadata
 * @description
 * The CommonMetadata component is a modular color selection system using the Compound Component pattern.
 * It provides a graphical picker for color selection with support for various color formats.
 *
 * @example
 * ```tsx
 * import { ColorPicker } from '@/components/ui';
 *
 * <ColorPicker
 *   class="custom-class"
 *   allowNoColor={false}
 *   showInput={true}
 *   color="#ff0000"
 *   onChange={(value) => console.log(value)}
 *   presets={["#ff0000", "#00ff00", "#0000ff"]}
 * />
 * ```
 */
export const CommonMetadata: Component<CommonMetadataProps> = props => {
    /**
     * Creates a signal for the notes.
     * @returns The notes signal.
     */
    const [notes, setNotes] = createSignal(untrack(() => props.item?.notes || ''));

    /**
     * Gets the library instance.
     * @returns The library instance.
     */
    const lib = useLibrary();

    /**
     * Creates an effect to update the notes when the item changes.
     */
    createEffect(() => {
        setNotes(props.item?.notes || '');
    });

    /**
     * Handles the notes change event.
     * @param val - The new notes value.
     */
    const handleNotesChange = (val: string) => {
        setNotes(val);
        if (props.item) {
            lib.updateItemNotes(props.item.id, val);
        }
    };

    /**
     * Handles the rating change event.
     * @param rating - The new rating value.
     */
    const handleRatingChange = (rating: number) => {
        if (props.item) {
            lib.updateItemRating(props.item.id, rating);
        }
    };

    /**
     * Renders the CommonMetadata component.
     * @returns The rendered CommonMetadata component.
     */
    return (
        <AccordionItem value="common">
            <AccordionHeader title="General Info" icon={<Info size={14} />} />
            <AccordionContent>
                <div class="inspector-field-group">
                    <label class="inspector-label">Name</label>
                    <Input value={props.item?.filename || ''} disabled />
                </div>

                <div class="inspector-field-group">
                    <label class="inspector-label">Rating</label>
                    <div class="inspector-rating-container">
                        <StarRating
                            rating={props.item?.rating || 0}
                            onChange={handleRatingChange}
                        />
                    </div>
                </div>

                <div class="inspector-grid">
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">Type</span>
                        <span class="inspector-meta-value inspector-meta-value-type">
                            <FileText size={10} />
                            {props.item?.format || props.item?.filename.split('.').pop()}
                        </span>
                    </div>
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">Size</span>
                        <span class="inspector-meta-value">
                            <HardDrive size={10} />
                            {props.item ? formatFileSize(props.item.size) : '-'}
                        </span>
                    </div>
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">Created</span>
                        <span class="inspector-meta-value">
                            <Calendar size={10} />
                            {props.item ? formatToDisplay(props.item.created_at) : '-'}
                        </span>
                    </div>
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">Modified</span>
                        <span class="inspector-meta-value">
                            <Calendar size={10} />
                            {props.item ? formatToDisplay(props.item.modified_at) : '-'}
                        </span>
                    </div>
                </div>

                <div class="inspector-field-group inspector-notes-group">
                    <label class="inspector-label">Notes</label>
                    <textarea
                        class="inspector-notes-input"
                        value={notes()}
                        onInput={e => handleNotesChange(e.currentTarget.value)}
                        placeholder="Add observations..."
                        rows={3}
                    />
                </div>
            </AccordionContent>
        </AccordionItem>
    );
};
