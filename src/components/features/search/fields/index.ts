import { SearchFieldHandler } from './types';
import { dateHandler } from './DateCriterionField';
import { folderHandler } from './FolderCriterionField';
import { numberHandler } from './NumberCriterionField';
import { ratingHandler } from './RatingCriterionField';
import { selectHandler } from './SelectCriterionField';
import { sizeHandler } from './SizeCriterionField';
import { tagsHandler } from './TagsCriterionField';
import { textHandler } from './TextCriterionField';

/**
 * Global registry mapping internal field type identifiers to their specialized logic handlers.
 * This registry allows the Advanced Search system to dynamically resolve validation,
 * processing, and UI rendering based on the type of search criterion selected.
 */
export const criterionHandlerRegistry: Record<string, SearchFieldHandler> = {
    date: dateHandler,
    folder: folderHandler,
    number: numberHandler,
    rating: ratingHandler,
    select: selectHandler,
    size: sizeHandler,
    tags: tagsHandler,
    text: textHandler
};

export * from './types';
