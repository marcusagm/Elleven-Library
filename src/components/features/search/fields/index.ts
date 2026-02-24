import { CriterionFieldRendererComponent } from './types';
import { DateCriterionField } from './DateCriterionField';
import { FolderCriterionField } from './FolderCriterionField';
import { NumberCriterionField } from './NumberCriterionField';
import { RatingCriterionField } from './RatingCriterionField';
import { SelectCriterionField } from './SelectCriterionField';
import { SizeCriterionField } from './SizeCriterionField';
import { TagsCriterionField } from './TagsCriterionField';
import { TextCriterionField } from './TextCriterionField';

export const criterionFieldRegistry: Record<string, CriterionFieldRendererComponent> = {
    date: DateCriterionField,
    folder: FolderCriterionField,
    number: NumberCriterionField,
    rating: RatingCriterionField,
    select: SelectCriterionField,
    size: SizeCriterionField,
    tags: TagsCriterionField,
    text: TextCriterionField
};

export * from './types';
