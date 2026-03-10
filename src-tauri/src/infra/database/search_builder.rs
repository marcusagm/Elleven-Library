//! Dynamic SQL query builder for advanced asset search.
//!
//! Ported and adapted from the legacy search engine to support the new
//! hexagonal architecture and final table schema.

use crate::core::models::search::{LogicalOperator, SearchCriterion, SearchGroup, SearchItem};
use sqlx::{QueryBuilder, Sqlite};

/// Builds the WHERE clause for an advanced search query.
///
/// # Arguments
///
/// * `group` - The root search group.
/// * `query_builder` - The SQLx QueryBuilder to append to.
pub fn build_search_where_clause<'a>(
    group: &'a SearchGroup,
    query_builder: &mut QueryBuilder<'a, Sqlite>,
) {
    query_builder.push(" (");

    let mut first = true;
    for item in &group.items {
        if !first {
            match group.logical_operator {
                LogicalOperator::And => {
                    query_builder.push(" AND ");
                }
                LogicalOperator::Or => {
                    query_builder.push(" OR ");
                }
            };
        }
        first = false;

        match item {
            SearchItem::Group(g) => build_search_where_clause(g, query_builder),
            SearchItem::Criterion(c) => build_search_criterion_clause(c, query_builder),
        }
    }

    if group.items.is_empty() {
        query_builder.push(" 1=1 ");
    }

    query_builder.push(") ");
}

/// Builds the WHERE clause for an advanced search query.
///
/// # Arguments
///
/// * `c` - The search criterion.
/// * `query_builder` - The SQLx QueryBuilder to append to.
fn build_search_criterion_clause<'a>(
    c: &'a SearchCriterion,
    query_builder: &mut QueryBuilder<'a, Sqlite>,
) {
    match c.key.as_str() {
        "name" | "path" | "format_type" | "family" => {
            query_builder.push(" a.");
            query_builder.push(&c.key);
            match c.operator.as_str() {
                "contains" => {
                    query_builder.push(" LIKE ");
                    query_builder.push_bind(format!("%{}%", c.value.as_str().unwrap_or("")));
                }
                "not_contains" => {
                    query_builder.push(" NOT LIKE ");
                    query_builder.push_bind(format!("%{}%", c.value.as_str().unwrap_or("")));
                }
                "equals" | "eq" | "exact" => {
                    query_builder.push(" = ");
                    query_builder.push_bind(c.value.as_str().unwrap_or(""));
                }
                "starts_with" => {
                    query_builder.push(" LIKE ");
                    query_builder.push_bind(format!("{}%", c.value.as_str().unwrap_or("")));
                }
                "ends_with" => {
                    query_builder.push(" LIKE ");
                    query_builder.push_bind(format!("%{}", c.value.as_str().unwrap_or("")));
                }
                _ => {
                    query_builder.push(" = 1 ");
                }
            }
        }
        "file_size" | "rating" | "width" | "height" | "duration_secs" => {
            // Note: width/height/duration_secs are in v2_asset_metadata_envelope (m)
            // but we'll assume the caller joins correctly if they use these keys.
            let prefix = if ["width", "height", "duration_secs"].contains(&c.key.as_str()) {
                "m."
            } else {
                "a."
            };

            query_builder.push(prefix);
            query_builder.push(&c.key);
            match c.operator.as_str() {
                "gt" | "greater_than" => {
                    query_builder.push(" > ");
                    query_builder.push_bind(c.value.as_i64().unwrap_or(0));
                }
                "lt" | "less_than" => {
                    query_builder.push(" < ");
                    query_builder.push_bind(c.value.as_i64().unwrap_or(0));
                }
                "eq" | "equals" => {
                    query_builder.push(" = ");
                    query_builder.push_bind(c.value.as_i64().unwrap_or(0));
                }
                "gte" | "greater_than_equals" => {
                    query_builder.push(" >= ");
                    query_builder.push_bind(c.value.as_i64().unwrap_or(0));
                }
                "lte" | "less_than_equals" => {
                    query_builder.push(" <= ");
                    query_builder.push_bind(c.value.as_i64().unwrap_or(0));
                }
                "between" => {
                    if let Some(arr) = c.value.as_array() {
                        if arr.len() == 2 {
                            query_builder.push(" BETWEEN ");
                            query_builder.push_bind(arr[0].as_i64().unwrap_or(0));
                            query_builder.push(" AND ");
                            query_builder.push_bind(arr[1].as_i64().unwrap_or(0));
                        } else {
                            query_builder.push(" = 1 ");
                        }
                    } else {
                        query_builder.push(" = 1 ");
                    }
                }
                _ => {
                    query_builder.push(" = 1 ");
                }
            }
        }
        "tags" => {
            // value can be a single tag name or an array of tag names
            match c.operator.as_str() {
                "contains" | "in" | "contains_any" => {
                    query_builder.push(" a.id IN (SELECT asset_id FROM asset_tags WHERE tag_id IN (SELECT id FROM tags WHERE name ");
                    if let Some(tags) = c.value.as_array() {
                        query_builder.push(" IN (");
                        let mut first = true;
                        for tag in tags {
                            if !first {
                                query_builder.push(", ");
                            }
                            query_builder.push_bind(tag.as_str().unwrap_or(""));
                            first = false;
                        }
                        query_builder.push(")) ");
                    } else {
                        query_builder.push(" = ");
                        query_builder.push_bind(c.value.as_str().unwrap_or(""));
                        query_builder.push(") ");
                    }
                    query_builder.push(") ");
                }
                "not_contains" => {
                    query_builder.push(" a.id NOT IN (SELECT asset_id FROM asset_tags WHERE tag_id IN (SELECT id FROM tags WHERE name ");
                    if let Some(tags) = c.value.as_array() {
                        query_builder.push(" IN (");
                        let mut first = true;
                        for tag in tags {
                            if !first {
                                query_builder.push(", ");
                            }
                            query_builder.push_bind(tag.as_str().unwrap_or(""));
                            first = false;
                        }
                        query_builder.push(")) ");
                    } else {
                        query_builder.push(" = ");
                        query_builder.push_bind(c.value.as_str().unwrap_or(""));
                        query_builder.push(") ");
                    }
                    query_builder.push(") ");
                }
                _ => {
                    query_builder.push(" 1=1 ");
                }
            }
        }
        "folder" => {
            let folder_path = c.value.as_str().unwrap_or("");
            match c.operator.as_str() {
                "is" | "equals" => {
                    query_builder.push(" a.folder_id IN (SELECT id FROM folders WHERE path = ");
                    query_builder.push_bind(folder_path);
                    query_builder.push(") ");
                }
                "in" | "recursive" => {
                    query_builder.push(" a.folder_id IN (WITH RECURSIVE subfolders AS (SELECT id FROM folders WHERE path = ");
                    query_builder.push_bind(folder_path);
                    query_builder.push(" UNION ALL SELECT f.id FROM folders f JOIN subfolders s ON f.parent_id = s.id) SELECT id FROM subfolders) ");
                }
                _ => {
                    query_builder.push(" 1=1 ");
                }
            }
        }
        "color" => {
            // CIE-76 Euclidean distance in LAB space.
            // Value: { "hex": "#FF5733", "threshold": 25.0 }
            let hex_color = c
                .value
                .get("hex")
                .and_then(|v| v.as_str())
                .unwrap_or("#000000");
            let threshold = c
                .value
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(25.0);

            match hex_to_lab(hex_color) {
                Ok((l, a, b)) => {
                    let threshold_squared = threshold * threshold;
                    query_builder.push(" a.id IN (SELECT DISTINCT asset_id FROM asset_colors WHERE ((lab_lightness - ");
                    query_builder.push_bind(l);
                    query_builder.push(") * (lab_lightness - ");
                    query_builder.push_bind(l);
                    query_builder.push(") + (lab_green_red - ");
                    query_builder.push_bind(a);
                    query_builder.push(") * (lab_green_red - ");
                    query_builder.push_bind(a);
                    query_builder.push(") + (lab_blue_yellow - ");
                    query_builder.push_bind(b);
                    query_builder.push(") * (lab_blue_yellow - ");
                    query_builder.push_bind(b);
                    query_builder.push(")) < ");
                    query_builder.push_bind(threshold_squared);
                    query_builder.push(") ");
                }
                Err(_) => {
                    query_builder.push(" 1=0 ");
                }
            }
        }
        _ => {
            query_builder.push(" 1=1 ");
        }
    }
}

/// Tests for the search builder.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::search::{LogicalOperator, SearchCriterion, SearchGroup, SearchItem};
    use serde_json::json;

    /// Tests the build_simple_where_clause function.
    #[test]
    fn test_build_simple_where_clause() {
        let criterion = SearchCriterion {
            id: "1".to_string(),
            key: "name".to_string(),
            operator: "contains".to_string(),
            value: json!("test"),
        };

        let group = SearchGroup {
            id: "root".to_string(),
            logical_operator: LogicalOperator::And,
            items: vec![SearchItem::Criterion(criterion)],
        };

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT * FROM assets WHERE ");
        build_search_where_clause(&group, &mut query_builder);

        let sql = query_builder.into_sql();
        assert!(sql.contains("a.name LIKE ?"));
    }

    /// Tests the build_search_where_clause function with nested groups.
    #[test]
    fn test_build_nested_where_clause() {
        let c1 = SearchCriterion {
            id: "1".to_string(),
            key: "name".to_string(),
            operator: "contains".to_string(),
            value: json!("a"),
        };
        let c2 = SearchCriterion {
            id: "2".to_string(),
            key: "format_type".to_string(),
            operator: "eq".to_string(),
            value: json!("image/jpeg"),
        };

        let nested_group = SearchGroup {
            id: "nested".to_string(),
            logical_operator: LogicalOperator::Or,
            items: vec![SearchItem::Criterion(c1), SearchItem::Criterion(c2)],
        };

        let root_group = SearchGroup {
            id: "root".to_string(),
            logical_operator: LogicalOperator::And,
            items: vec![SearchItem::Group(nested_group)],
        };

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT * FROM assets WHERE ");
        build_search_where_clause(&root_group, &mut query_builder);

        let sql = query_builder.into_sql();
        assert!(sql.contains("a.name LIKE ?"));
        assert!(sql.contains("OR"));
        assert!(sql.contains("a.format_type = ?"));
    }

    /// Tests the build_search_where_clause function with color proximity.
    #[test]
    fn test_color_search_proximity_sql() {
        let criterion = SearchCriterion {
            id: "color1".to_string(),
            key: "color".to_string(),
            operator: "proximity".to_string(),
            value: json!({ "hex": "#FF0000", "threshold": 10.0 }),
        };

        let group = SearchGroup {
            id: "root".to_string(),
            logical_operator: LogicalOperator::And,
            items: vec![SearchItem::Criterion(criterion)],
        };

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT * FROM assets WHERE ");
        build_search_where_clause(&group, &mut query_builder);

        let sql = query_builder.into_sql();
        assert!(sql.contains(
            "a.id IN (SELECT DISTINCT asset_id FROM asset_colors WHERE ((lab_lightness - ?"
        ));
        assert!(sql.contains("< ?"));
    }
}

/// Converts a hexadecimal color string to CIE-LAB components.
///
/// Supports formats: "#RRGGBB" and "RRGGBB".
///
/// # Errors
///
/// Returns an error if the hex string is invalid.
fn hex_to_lab(
    hex_color: &str,
) -> Result<(f64, f64, f64), Box<dyn std::error::Error + Send + Sync>> {
    let hex_trimmed = hex_color.trim_start_matches('#');
    if hex_trimmed.len() != 6 {
        return Err(format!("Invalid hex color length: {}", hex_color).into());
    }
    let red = u8::from_str_radix(&hex_trimmed[0..2], 16)?;
    let green = u8::from_str_radix(&hex_trimmed[2..4], 16)?;
    let blue = u8::from_str_radix(&hex_trimmed[4..6], 16)?;
    let srgb_color = palette::Srgb::new(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
    );
    let lab_color: palette::Lab = palette::IntoColor::into_color(srgb_color);
    Ok((lab_color.l as f64, lab_color.a as f64, lab_color.b as f64))
}
