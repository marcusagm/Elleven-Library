import { Component, For, createSignal, Show, onCleanup, createEffect } from "solid-js";
import { Portal } from "solid-js/web";
import { X } from "lucide-solid";
import { computePosition, flip, shift, offset, autoUpdate } from "@floating-ui/dom";
import "./tag-input.css";

export interface TagOption {
    id: string | number;
    label: string;
    color?: string;
}

interface TagInputProps {
    value: TagOption[];
    onChange: (tags: TagOption[]) => void;
    placeholder?: string;
    suggestions?: TagOption[]; // All available tags for autocomplete
    onCreate?: (name: string) => void; // Optional: create new tag
}

export const TagInput: Component<TagInputProps> = (props) => {
    const [inputValue, setInputValue] = createSignal("");
    const [showSuggestions, setShowSuggestions] = createSignal(false);
    let inputContainerRef: HTMLDivElement | undefined;
    let dropdownRef: HTMLDivElement | undefined;
    let cleanupFloating: (() => void) | undefined;

    // Filter suggestions based on input
    const filteredSuggestions = () => {
        const input = inputValue().toLowerCase();
        if (!input) return [];
        return (props.suggestions || [])
            .filter(t => t.label.toLowerCase().includes(input))
            .filter(t => !props.value.some(v => v.id === t.id)); // Exclude already selected
    };

    const updatePosition = () => {
        if (inputContainerRef && dropdownRef) {
            computePosition(inputContainerRef, dropdownRef, {
                placement: "bottom-start",
                middleware: [
                    offset(4),
                    flip(),
                    shift({ padding: 8 })
                ],
            }).then(({ x, y }) => {
                if (dropdownRef) {
                    Object.assign(dropdownRef.style, {
                        left: `${x}px`,
                        top: `${y}px`,
                        position: 'absolute',
                        width: `${inputContainerRef?.offsetWidth}px` // Match width
                    });
                }
            });
        }
    };

    // Effect to run setup when visibility changes is implied by reactivity in solid
    // We can just call setupFloating in an effect or whenever showSuggestions/filtered changes.
    // However, since dropdownRef is conditional (Show), we need to be careful.
    
    // Better approach: use a ref callback or simple effect if ref exists.
    // In Solid, references are assigned on mount. 
    
    // Let's rely on an effect that watches the visibility condition
    createEffect(() => {
        const visible = showSuggestions() && inputValue() && filteredSuggestions().length > 0;
        if (visible) {
             // Wait for DOM render of dropdown
             setTimeout(() => {
                 if (inputContainerRef && dropdownRef) {
                     updatePosition();
                     if (!cleanupFloating) {
                         cleanupFloating = autoUpdate(inputContainerRef, dropdownRef, updatePosition);
                     }
                 }
             }, 0);
        } else {
             if (cleanupFloating) {
                 cleanupFloating();
                 cleanupFloating = undefined;
             }
        }
    });

    onCleanup(() => {
        if (cleanupFloating) cleanupFloating();
    });

    const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === "Enter") {
            e.preventDefault();
            const val = inputValue().trim();
            if (!val) return;
            
            const relevant = filteredSuggestions();
            const exactMatch = relevant.find(t => t.label.toLowerCase() === val.toLowerCase());
            
            if (exactMatch) {
                addTag(exactMatch);
            } else if (props.onCreate) {
                props.onCreate(val);
                setInputValue("");
                setShowSuggestions(false);
            }
        } else if (e.key === "Backspace" && !inputValue() && props.value.length > 0) {
            // Remove last tag
            const newVal = [...props.value];
            newVal.pop();
            props.onChange(newVal);
        } else if (e.key === "Escape") {
            setShowSuggestions(false);
        }
    };

    const addTag = (tag: TagOption) => {
        props.onChange([...props.value, tag]);
        setInputValue("");
        setShowSuggestions(false);
    };

    const removeTag = (id: string | number) => {
        props.onChange(props.value.filter(t => t.id !== id));
    };

    return (
        <div class="tag-input-container" ref={inputContainerRef}>
            <div class="tag-input-content">
                <For each={props.value}>
                    {(tag) => (
                        <div 
                            class="tag-chip" 
                            style={tag.color ? { "background-color": tag.color, "color": "white" } : {}}
                        >
                            <span>{tag.label}</span>
                            <button onClick={() => removeTag(tag.id)} class="tag-remove-btn">
                                <X size={12} />
                            </button>
                        </div>
                    )}
                </For>
                <input
                    type="text"
                    value={inputValue()}
                    onInput={(e) => {
                        setInputValue(e.currentTarget.value);
                        setShowSuggestions(true);
                    }}
                    onKeyDown={handleKeyDown}
                    onBlur={() => setTimeout(() => setShowSuggestions(false), 200)}
                    onFocus={() => setShowSuggestions(true)}
                    placeholder={props.value.length === 0 ? props.placeholder : ""}
                    class="tag-input-field"
                />
            </div>
            
            <Show when={showSuggestions() && inputValue() && filteredSuggestions().length > 0}>
                <Portal>
                    <div 
                        class="tag-suggestions-dropdown" 
                        ref={dropdownRef}
                        // Stop propagation of mousedown to prevent blur of input
                         onMouseDown={(e) => e.preventDefault()}
                    >
                        <For each={filteredSuggestions()}>
                            {(suggestion) => (
                                <div 
                                    class="tag-suggestion-item" 
                                    onClick={() => addTag(suggestion)}
                                >
                                    {suggestion.label}
                                </div>
                            )}
                        </For>
                    </div>
                </Portal>
            </Show>
        </div>
    );
};
