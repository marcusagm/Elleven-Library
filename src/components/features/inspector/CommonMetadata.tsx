import { Component } from "solid-js";
import { Info, FileText, Calendar, HardDrive } from "lucide-solid";
import { AccordionItem } from "../../ui/Accordion";
import { Input } from "../../ui/Input";
import { StarRating } from "./StarRating";
import { ImageItem } from "../../../core/store/appStore"; // Importing interface
import "./inspector.css";

interface CommonMetadataProps {
    item: ImageItem | null;
    rating?: number;
    onRatingChange?: (r: number) => void;
    notes?: string;
    onNotesChange?: (n: string) => void;
}

export const CommonMetadata: Component<CommonMetadataProps> = (props) => {
    return (
        <AccordionItem 
            value="common" 
            title="General Info" 
            defaultOpen 
            icon={<Info size={14} />}
        >
            <div class="inspector-field-group">
                <label class="inspector-label">Name</label>
                <Input value={props.item?.filename || ""} disabled />
            </div>

            <div class="inspector-field-group">
               <label class="inspector-label">Rating</label>
               <div class="inspector-rating-container">
                   <StarRating 
                        rating={props.rating || 0} 
                        onChange={props.onRatingChange} 
                   />
               </div>
            </div>

            <div class="inspector-grid">
                <div class="inspector-meta-item">
                    <span class="inspector-meta-label">Type</span>
                    <span class="inspector-meta-value">
                        <FileText size={10} />
                        {props.item?.filename.split('.').pop()?.toUpperCase()}
                    </span>
                </div>
                <div class="inspector-meta-item">
                     <span class="inspector-meta-label">Size</span>
                     <span class="inspector-meta-value">
                        <HardDrive size={10} />
                        -- MB
                     </span>
                </div>
                 <div class="inspector-meta-item">
                     <span class="inspector-meta-label">Created</span>
                     <span class="inspector-meta-value">
                        <Calendar size={10} />
                        --/--/--
                     </span>
                </div>
                 <div class="inspector-meta-item">
                     <span class="inspector-meta-label">Modified</span>
                     <span class="inspector-meta-value">
                        <Calendar size={10} />
                        --/--/--
                     </span>
                </div>
            </div>

            <div class="inspector-field-group" style="margin-top: 12px;">
                <label class="inspector-label">Notes</label>
                <textarea 
                    class="inspector-notes-input"
                    value={props.notes || ""}
                    onChange={(e) => props.onNotesChange?.(e.currentTarget.value)}
                    placeholder="Add observations..."
                    rows={3}
                />
            </div>
        </AccordionItem>
    );
};
