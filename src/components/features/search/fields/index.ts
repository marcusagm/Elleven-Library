import { SearchFieldHandler } from './types';
import { dateHandler } from './DateCriterionField';
import { folderHandler } from './FolderCriterionField';
import { numberHandler } from './NumberCriterionField';
import { ratingHandler } from './RatingCriterionField';
import { selectHandler } from './SelectCriterionField';
import { sizeHandler } from './SizeCriterionField';
import { tagsHandler } from './TagsCriterionField';
import { textHandler } from './TextCriterionField';

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
