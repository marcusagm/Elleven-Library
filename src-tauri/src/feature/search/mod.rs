/// Module for advanced search operations.
///
/// This module contains the implementation of the SearchQueryHandler, which is responsible for
/// performing advanced searches based on the provided criteria.
///
/// # Submodules
///
/// * `query_handler` - The query handler for advanced search operations.
///
/// # Exports
///
/// * `SearchQueryHandler` - The query handler for advanced search operations.
///
/// # Example
///
/// ```rust
/// use crate::feature::search::SearchQueryHandler;
/// use crate::core::models::SearchCriteria;
/// use crate::core::models::PageParams;
/// use crate::core::models::AssetSummaryDto;
/// use crate::core::models::AppResult;
/// use std::sync::Arc;
/// use crate::core::repository::AssetQueryHandler;
///
/// let search_query_handler = SearchQueryHandler::new(Arc::new(AssetQueryHandler::new()));
/// let criteria = SearchCriteria::new();
/// let page = PageParams::new();
/// let result: AppResult<Vec<AssetSummaryDto>> = search_query_handler.search(criteria, page).await;
/// ```
pub mod query_handler;

pub use query_handler::SearchQueryHandler;
