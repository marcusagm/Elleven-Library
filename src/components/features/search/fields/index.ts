import { coreCriterionHandlerRegistry } from '../../../../core/store/filter/handlers';
import { SearchFieldHandler } from './types';

import { ColorCriterionField } from './ColorCriterionField';
import { DateCriterionField } from './DateCriterionField';
import { FolderCriterionField } from './FolderCriterionField';
import { NumberCriterionField } from './NumberCriterionField';
import { RatingCriterionField } from './RatingCriterionField';
import { SelectCriterionField } from './SelectCriterionField';
import { SizeCriterionField } from './SizeCriterionField';
import { TagsCriterionField } from './TagsCriterionField';
import { TextCriterionField } from './TextCriterionField';

/**
 * Global registry mapping internal field type identifiers to their specialized logic handlers.
 * This registry allows the Advanced Search system to dynamically resolve validation,
 * processing, and UI rendering based on the type of search criterion selected.
 *
 * @type {Record<string, SearchFieldHandler>}
 */
export const criterionHandlerRegistry: Record<string, SearchFieldHandler> = {
    color: { component: ColorCriterionField, ...coreCriterionHandlerRegistry.color },
    date: { component: DateCriterionField, ...coreCriterionHandlerRegistry.date },
    folder: { component: FolderCriterionField, ...coreCriterionHandlerRegistry.folder },
    number: { component: NumberCriterionField, ...coreCriterionHandlerRegistry.number },
    rating: { component: RatingCriterionField, ...coreCriterionHandlerRegistry.rating },
    select: { component: SelectCriterionField, ...coreCriterionHandlerRegistry.select },
    size: { component: SizeCriterionField, ...coreCriterionHandlerRegistry.size },
    tags: { component: TagsCriterionField, ...coreCriterionHandlerRegistry.tags },
    text: { component: TextCriterionField, ...coreCriterionHandlerRegistry.text }
};

export * from './types';
