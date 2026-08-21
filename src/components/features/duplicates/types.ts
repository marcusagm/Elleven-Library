export interface DuplicateCandidate {
    id: string;
    name: string;
    size: string;
    dimensions: string;
    score: number;
    path: string;
    format: string;
    createdAt: string;
    updatedAt: string;
    tags: string[];
    notes?: string;
    isFavorite: boolean;
    thumbnailUrl?: string;
    mediaType?: string;
}

export interface DuplicateGroup {
    id: string;
    type: 'exact' | 'visual' | 'derived';
    confidence: number;
    candidates: DuplicateCandidate[];
}
