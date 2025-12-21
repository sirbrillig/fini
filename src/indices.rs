use crate::storage::TaskStorage;
use crate::util::{get_task_id_by_index_from_list, get_visible_items};

pub fn get_id_for_index(
    storage: &dyn TaskStorage,
    index: usize,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let visible = get_visible_items(storage)?;
    Ok(get_task_id_by_index_from_list(index, &visible))
}

pub fn get_ids_for_indices(
    storage: &dyn TaskStorage,
    indices: Vec<usize>,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let visible = get_visible_items(storage)?;
    Ok(indices
        .iter()
        .filter_map(|index| get_task_id_by_index_from_list(*index, &visible))
        .collect())
}

pub fn get_id_for_title(
    storage: &dyn TaskStorage,
    title: &str,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let visible = get_visible_items(storage)?;
    let item = visible.iter().find(|t| t.title == title);
    match item {
        Some(task) => Ok(Some(task.id)),
        None => Ok(None),
    }
}
