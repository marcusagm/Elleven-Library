use serde::{Deserialize, Serialize};

/// Logical operator for joining search criteria groups.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LogicalOperator {
    /// Matches all criteria in the group.
    And,
    /// Matches any of the criteria in the group.
    Or,
}

/// Represents a set of search criteria that can be nested.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchCriteria {
    /// A unique identifier for the criteria set (useful for frontend tracking).
    pub id: String,
    /// The root group of search items.
    pub root_group: SearchGroup,
}

/// A group of search items joined by a logical operator.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchGroup {
    /// A unique identifier for the group.
    pub id: String,
    /// The logical operator used to join the items in this group.
    pub logical_operator: LogicalOperator,
    /// The list of items (nested groups or specific criteria) in this group.
    pub items: Vec<SearchItem>,
}

/// A single item in a search group, either another group or a concrete criterion.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SearchItem {
    /// A nested group of criteria.
    Group(SearchGroup),
    /// A concrete search criterion (e.g., name equals "foo").
    Criterion(SearchCriterion),
}

/// A concrete search criterion.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchCriterion {
    /// A unique identifier for the criterion.
    pub id: String,
    /// The field name to filter on (e.g., "name", "size", "tag").
    pub key: String,
    /// The operator to use (e.g., "equals", "contains", "gt").
    pub operator: String,
    /// The value to compare against.
    pub value: serde_json::Value,
}
