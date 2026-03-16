//! Core domain models for the application.

pub mod asset;
pub mod search;
pub mod smart_folder;

pub use asset::{Asset, AssetColor, AssetFilter, AssetState, AssetSummaryDto, Folder, LibraryStats, PageParams, PaginatedAssetsDto, Tag, TagCount, FolderCount};
pub use search::{LogicalOperator, SearchCriteria, SearchCriterion, SearchGroup, SearchItem};
pub use smart_folder::SmartFolder;
