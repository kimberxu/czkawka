use std::cmp::Ordering;
use std::rc::Rc;
use slint::{ComponentHandle, Model, VecModel};
use crate::{AdvancedSelectionCriterion, Callabler, GuiState, MainWindow};
use crate::connect_row_selection::checker::change_number_of_enabled_items;
use crate::common::connect_i32_into_u64;

pub fn connect_advanced_selection(app: &MainWindow) {
    connect_select_advanced_custom_path(app);
    connect_select_advanced_group(app);
}

fn connect_select_advanced_custom_path(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_select_advanced_custom_path(move |path, include_subdirs, mode, uncheck_baseline| {
        let app = a.upgrade().unwrap();
        select_by_path(&app, &path, include_subdirs, mode, uncheck_baseline);
    });
}

pub fn select_by_path(app: &MainWindow, filter_path: &str, include_subdirs: bool, mode: i32, uncheck_baseline: bool) {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();
    let is_header_mode = active_tab.get_is_header_mode();

    // Requires groups (header mode) for safe "select other" logic
    if !is_header_mode {
        return;
    }

    let mut checked_count_change = 0i64;
    let row_count = model.row_count();
    let mut old_data = model.iter().collect::<Vec<_>>();
    
    // Normalize filter path
    let filter_path = filter_path.replace('\\', "/");
    let filter_path = if cfg!(target_os = "windows") {
        filter_path.trim_end_matches('/').to_lowercase()
    } else {
        filter_path.trim_end_matches('/').to_string()
    };

    // Find groups
    let mut headers_idx = Vec::new();
    for (idx, item) in old_data.iter().enumerate() {
        if item.header_row {
            headers_idx.push(idx);
        }
    }
    headers_idx.push(row_count);

    for i in 0..(headers_idx.len() - 1) {
        let start = headers_idx[i] + 1;
        let end = headers_idx[i + 1];
        
        if start >= end {
            continue;
        }

        // Check group content
        let mut group_has_match = false;
        let mut indices_match = Vec::new();
        let mut indices_not_match = Vec::new();

        for idx in start..end {
            let path = old_data[idx].val_str.iter().nth(path_idx).unwrap_or_default();
            let row_path = path.replace('\\', "/");
            let row_path = if cfg!(target_os = "windows") {
                row_path.trim_end_matches('/').to_lowercase()
            } else {
                row_path.trim_end_matches('/').to_string()
            };

            let is_match = if include_subdirs {
                if row_path == filter_path {
                    true
                } else if row_path.starts_with(&filter_path) {
                    row_path.as_bytes().get(filter_path.len()) == Some(&b'/')
                } else {
                    false
                }
            } else {
                row_path == filter_path
            };

            if is_match {
                group_has_match = true;
                indices_match.push(idx);
            } else {
                indices_not_match.push(idx);
            }
        }

        match mode {
            0 => { // Select This (Check Match, Uncheck Not Match)
                if group_has_match {
                    for idx in indices_match {
                        if !old_data[idx].checked {
                            old_data[idx].checked = true;
                            checked_count_change += 1;
                        }
                    }
                    if uncheck_baseline {
                        for idx in indices_not_match {
                            if old_data[idx].checked {
                                old_data[idx].checked = false;
                                checked_count_change -= 1;
                            }
                        }
                    }
                }
            },
            1 => { // Select Other (Check Not Match, Uncheck Match)
                // SAFETY: Only if we have a match inside to keep!
                if group_has_match {
                    for idx in indices_not_match {
                        if !old_data[idx].checked {
                            old_data[idx].checked = true;
                            checked_count_change += 1;
                        }
                    }
                    if uncheck_baseline {
                        for idx in indices_match {
                            if old_data[idx].checked {
                                old_data[idx].checked = false;
                                checked_count_change -= 1;
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    
    if checked_count_change != 0 {
        let new_model = Rc::new(VecModel::from(old_data));
        active_tab.set_tool_model(app, new_model.into());
        change_number_of_enabled_items(app, active_tab, checked_count_change);
    }
}


fn connect_select_advanced_group(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_select_advanced_group(move |criteria_model| {
        let app = a.upgrade().unwrap();
        let criteria: Vec<AdvancedSelectionCriterion> = criteria_model.iter().collect();
        select_by_criteria(&app, &criteria);
    });
}

fn select_by_criteria(app: &MainWindow, criteria: &[AdvancedSelectionCriterion]) {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    
    if !active_tab.get_is_header_mode() {
        return; // Only works for tools with groups
    }

    let row_count = model.row_count();
    if row_count == 0 {
        return;
    }

    let mut old_data = model.iter().collect::<Vec<_>>();
    let mut checked_count_change = 0i64;

    // Find headers
    let mut headers_idx = Vec::new();
    for (idx, item) in old_data.iter().enumerate() {
        if item.header_row {
            headers_idx.push(idx);
        }
    }
    headers_idx.push(row_count);

    let path_idx = active_tab.get_str_path_idx();
    let name_idx = active_tab.get_str_name_idx();
    let size_idx = active_tab.get_int_size_opt_idx().unwrap_or(0); // Only if size enabled
    let date_idx = active_tab.get_int_modification_date_idx();

    // Filter enabled criteria
    let active_criteria: Vec<&AdvancedSelectionCriterion> = criteria.iter().filter(|c| c.enabled).collect();

    for i in 0..(headers_idx.len() - 1) {
        let start = headers_idx[i] + 1;
        let end = headers_idx[i + 1];
        
        if start >= end {
            continue;
        }

        let mut indices: Vec<usize> = (start..end).collect();

        // Sort indices based on criteria
        indices.sort_by(|&a_idx, &b_idx| {
            let row_a = &old_data[a_idx];
            let row_b = &old_data[b_idx];

            for criterion in &active_criteria {
                let ordering = match criterion.id {
                    0 => { // Modification Date
                        let val_a = connect_i32_into_u64(row_a.val_int.row_data(date_idx).unwrap_or_default(), row_a.val_int.row_data(date_idx+1).unwrap_or_default());
                        let val_b = connect_i32_into_u64(row_b.val_int.row_data(date_idx).unwrap_or_default(), row_b.val_int.row_data(date_idx+1).unwrap_or_default());
                        val_a.cmp(&val_b)
                    },
                    1 => Ordering::Equal, // Creation Date not supported
                    2 => { // Size
                        let val_a = connect_i32_into_u64(row_a.val_int.row_data(size_idx).unwrap_or_default(), row_a.val_int.row_data(size_idx+1).unwrap_or_default());
                        let val_b = connect_i32_into_u64(row_b.val_int.row_data(size_idx).unwrap_or_default(), row_b.val_int.row_data(size_idx+1).unwrap_or_default());
                        val_a.cmp(&val_b)
                    },
                    3 => { // Filename Length
                        let name_a = &row_a.val_str.row_data(name_idx).unwrap_or_default();
                        let name_b = &row_b.val_str.row_data(name_idx).unwrap_or_default();
                        name_a.len().cmp(&name_b.len())
                    },
                    4 => { // Path Length
                        let path_a = &row_a.val_str.row_data(path_idx).unwrap_or_default();
                        let path_b = &row_b.val_str.row_data(path_idx).unwrap_or_default();
                        path_a.len().cmp(&path_b.len())
                    },
                    _ => Ordering::Equal,
                };

                if ordering != Ordering::Equal {
                    return if criterion.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    };
                }
            }
            Ordering::Equal
        });

        // The first item in sorted list is the "Best Match".
        // We CHECK the best match and UNCHECK others.
        // Wait, normally we Keep the best match and Check others for deletion.
        // But logic in `select_by_size_date` (Czkawka) was: find max, check it.
        // If I follow Czkawka: "Select" means "Check".
        // So I check the Best Match.
        
        let best_match_idx = indices[0];
        
        // Uncheck all in group
        for &idx in &indices {
            if old_data[idx].checked {
                old_data[idx].checked = false;
                checked_count_change -= 1;
            }
        }
        
        // Check best match
        if !old_data[best_match_idx].checked {
            old_data[best_match_idx].checked = true;
            checked_count_change += 1;
        }
    }

    if checked_count_change != 0 {
        // Update model
        let new_model = Rc::new(VecModel::from(old_data));
        active_tab.set_tool_model(app, new_model.into());
        change_number_of_enabled_items(app, active_tab, checked_count_change);
    }
}
