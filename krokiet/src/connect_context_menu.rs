use std::path::Path;
use std::rc::Rc;

use arboard::Clipboard;
use slint::{ComponentHandle, Model, SharedString, VecModel};

use crate::connect_row_selection::checker::change_number_of_enabled_items;
use crate::{Callabler, GuiState, MainWindow};

pub(crate) fn connect_context_menu(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_context_open_file(move |idx| {
        let app = a.upgrade().expect("Failed to upgrade app");
        open_item(&app, idx, false);
    });

    let a = app.as_weak();
    app.global::<Callabler>().on_context_open_folder(move |idx| {
        let app = a.upgrade().expect("Failed to upgrade app");
        open_item(&app, idx, true);
    });

    let a = app.as_weak();
    app.global::<Callabler>().on_context_select_in_folder(move |idx| {
        let app = a.upgrade().expect("Failed to upgrade app");
        modify_selection_by_folder(&app, idx, true);
    });

    let a = app.as_weak();
    app.global::<Callabler>().on_context_select_other_folders(move |idx| {
        let app = a.upgrade().expect("Failed to upgrade app");
        modify_selection_by_folder(&app, idx, false);
    });

    let a = app.as_weak();
    app.global::<Callabler>().on_context_protect_folder(move |idx| {
        let app = a.upgrade().expect("Failed to upgrade app");
        protect_folder(&app, idx);
    });

    connect_copy_path(app);
    connect_select_tree_request(app);
    connect_select_other_request(app);
    connect_select_tree_confirm(app);
    connect_select_other_confirm(app);
}

fn open_item(app: &MainWindow, idx: i32, parent_folder: bool) {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();

    if let Some(item) = model.row_data(idx as usize) {
        if let Some(path_str) = item.val_str.iter().nth(path_idx) {
            let path_str = path_str.as_str();
            if parent_folder {
                if let Some(parent) = Path::new(path_str).parent() {
                    let _ = open::that(parent);
                }
            } else {
                let _ = open::that(path_str);
            }
        }
    }
}

fn modify_selection_by_folder(app: &MainWindow, idx: i32, same_folder: bool) {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();
    let is_header_mode = active_tab.get_is_header_mode();

    let target_folder = if let Some(item) = model.row_data(idx as usize) {
        if let Some(path_str) = item.val_str.iter().nth(path_idx) {
             if let Some(parent) = Path::new(path_str.as_str()).parent() {
                 parent.to_path_buf()
             } else {
                 return;
             }
        } else { return; }
    } else { return; };

    let mut old_data = model.iter().collect::<Vec<_>>();
    let mut changed = false;

    for item in old_data.iter_mut() {
        if is_header_mode && item.header_row {
            continue;
        }
        if let Some(path_str) = item.val_str.iter().nth(path_idx) {
             if let Some(parent) = Path::new(path_str.as_str()).parent() {
                 let is_same = parent == target_folder;
                 if same_folder {
                     // Select items in SAME folder
                     if is_same && !item.checked {
                         item.checked = true;
                         changed = true;
                     }
                 } else {
                     // Select items in OTHER folders
                     if !is_same && !item.checked {
                         item.checked = true;
                         changed = true;
                     }
                 }
             }
        }
    }

    if changed {
        active_tab.set_tool_model(app, slint::ModelRc::new(slint::VecModel::from(old_data)));
    }
}

fn protect_folder(app: &MainWindow, idx: i32) {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();
    let is_header_mode = active_tab.get_is_header_mode();

    let target_folder = if let Some(item) = model.row_data(idx as usize) {
        if let Some(path_str) = item.val_str.iter().nth(path_idx) {
             if let Some(parent) = Path::new(path_str.as_str()).parent() {
                 parent.to_path_buf()
             } else {
                 return;
             }
        } else { return; }
    } else { return; };

    let mut old_data = model.iter().collect::<Vec<_>>();
    let mut changed = false;

    for item in old_data.iter_mut() {
        if is_header_mode && item.header_row {
            continue;
        }
        if let Some(path_str) = item.val_str.iter().nth(path_idx) {
             if let Some(parent) = Path::new(path_str.as_str()).parent() {
                 if parent == target_folder && item.checked {
                     item.checked = false; // Uncheck
                     changed = true;
                 }
             }
        }
    }

    if changed {
        active_tab.set_tool_model(app, slint::ModelRc::new(slint::VecModel::from(old_data)));
    }
}

fn get_path_from_idx(app: &MainWindow, idx: usize) -> Option<String> {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    let row = model.row_data(idx)?;
    
    let path_idx = active_tab.get_str_path_idx();
    let name_idx = active_tab.get_str_name_idx();

    if idx >= model.row_count() {
        return None;
    }

    let path = row.val_str.iter().nth(path_idx).unwrap_or_default();
    let name = row.val_str.iter().nth(name_idx).unwrap_or_default();
    
    let full_path = if name.is_empty() {
        path.to_string()
    } else {
        format!("{}/{}", path, name)
    };
    
    Some(full_path)
}

fn connect_copy_path(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_copy_path(move |idx| {
        let app = a.upgrade().unwrap();
        if let Some(path) = get_path_from_idx(&app, idx as usize) {
            let path = Path::new(&path);
            let parent = path.parent().unwrap_or(path);
            let parent_str = parent.to_string_lossy().to_string();

            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(parent_str) {
                        eprintln!("Failed to copy to clipboard: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to initialize clipboard: {}", e),
            }
        }
    });
}


fn connect_select_tree_request(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_select_tree_request(move |idx| {
        let app = a.upgrade().unwrap();
        if let Some(path_str) = get_path_from_idx(&app, idx as usize) {
            let path = Path::new(&path_str);
            let mut ancestors: Vec<SharedString> = Vec::new();
            
            for ancestor in path.ancestors() {
                if let Some(s) = ancestor.to_str() {
                    if !s.is_empty() {
                         ancestors.push(SharedString::from(s));
                    }
                }
            }
            
            let model = Rc::new(VecModel::from(ancestors));
            app.global::<GuiState>().set_directory_selection_model(model.into());
            app.invoke_show_directory_selection_popup(false);
        }
    });
}

fn connect_select_other_request(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_select_other_request(move |idx| {
        let app = a.upgrade().unwrap();
        if let Some(path_str) = get_path_from_idx(&app, idx as usize) {
            let path = Path::new(&path_str);
            let mut ancestors: Vec<SharedString> = Vec::new();
            for ancestor in path.ancestors() {
                if let Some(s) = ancestor.to_str() {
                    if !s.is_empty() {
                         ancestors.push(SharedString::from(s));
                    }
                }
            }
            let model = Rc::new(VecModel::from(ancestors));
            app.global::<GuiState>().set_directory_selection_model(model.into());
            app.invoke_show_directory_selection_popup(true);
        }
    });
}

fn connect_select_tree_confirm(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_select_tree_confirm(move |path_to_select_str| {
        let app = a.upgrade().unwrap();
        let path_to_select = path_to_select_str.as_str();
        select_by_path(&app, path_to_select, true);
    });
}

fn connect_select_other_confirm(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_select_other_confirm(move |path_to_avoid_str| {
        let app = a.upgrade().unwrap();
        let path_to_avoid = path_to_avoid_str.as_str();
        select_by_path(&app, path_to_avoid, false);
    });
}

fn select_by_path(app: &MainWindow, filter_path: &str, select_inside: bool) {
    let active_tab = app.global::<GuiState>().get_active_tab();
    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();
    let is_header_mode = active_tab.get_is_header_mode();
    
    // If not in header mode (no groups), we can't safely apply "select other" logic without risking deleting everything.
    // Assuming this tool is only for duplicates/similar which have groups.
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

        // Check if group has ANY file inside filter_path
        let mut group_has_inside = false;
        let mut indices_inside = Vec::new();
        let mut indices_outside = Vec::new();

        for idx in start..end {
            let path = old_data[idx].val_str.iter().nth(path_idx).unwrap_or_default();
            let row_path = path.replace('\\', "/");
            let row_path = if cfg!(target_os = "windows") {
                row_path.trim_end_matches('/').to_lowercase()
            } else {
                row_path.trim_end_matches('/').to_string()
            };
            
            let is_inside = if row_path == filter_path {
                true
            } else if row_path.starts_with(&filter_path) {
                row_path.as_bytes().get(filter_path.len()) == Some(&b'/')
            } else {
                false
            };


            if is_inside {
                group_has_inside = true;
                indices_inside.push(idx);
            } else {
                indices_outside.push(idx);
            }
        }

        // Apply logic
        if select_inside {
            // "Select duplicates in THIS directory"
            // Check inside, Uncheck outside (usually we want to keep one, so maybe Uncheck outside is redundant if we assume user wants to delete inside?)
            // "Select" usually means "Mark for deletion".
            // So "Select duplicates in THIS directory" -> Delete files in THIS directory.
            for idx in indices_inside {
                if !old_data[idx].checked {
                    old_data[idx].checked = true;
                    checked_count_change += 1;
                }
            }
            // Optional: Uncheck outside? Or leave them? Czkawka usually unchecks others to ensure we only delete what is asked.
            for idx in indices_outside {
                if old_data[idx].checked {
                    old_data[idx].checked = false;
                    checked_count_change -= 1;
                }
            }
        } else {
            // "Select duplicates in OTHER directories" (Select Other)
            // Goal: Keep files in THIS directory, delete others.
            // Safety: Only apply if the group HAS a file in THIS directory.
            if group_has_inside {
                for idx in indices_outside {
                    if !old_data[idx].checked {
                        old_data[idx].checked = true;
                        checked_count_change += 1;
                    }
                }
                for idx in indices_inside {
                    if old_data[idx].checked {
                        old_data[idx].checked = false;
                        checked_count_change -= 1;
                    }
                }
            }
        }
    }
    
    if checked_count_change != 0 {
        let new_model = Rc::new(VecModel::from(old_data));
        active_tab.set_tool_model(app, new_model.into());
        change_number_of_enabled_items(app, active_tab, checked_count_change);
    }
}

