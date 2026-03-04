export interface TechItem {
    name: string;
    description: string;
    url: string;
}

export interface TechCategory {
    title: string;
    items: TechItem[];
}
