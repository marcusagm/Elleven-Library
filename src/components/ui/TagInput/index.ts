/**
 * TagInput component
 *
 * @module TagInput
 * @description
 * The TagInput component is a specialized Input component that allows the user to enter tags.
 * It provides controls for adding/removing tags and filtering suggestions.
 *
 * @example
 * <TagInput
 *   value={['tag1', 'tag2']}
 *   onChange={handleChange}
 *   placeholder="Enter tags"
 *   suggestions={['tag1', 'tag2', 'tag3']}
 *   onCreate={handleCreate}
 *   disabled={false}
 *   max={10}
 *   class="tag-input"
 * />
 */
export * from './TagInput';
export * from './TagChip';
export * from './TagSuggestions';
export * from './types';
