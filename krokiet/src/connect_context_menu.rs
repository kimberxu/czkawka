use std::path::Path;
use std::rc::Rc;

use arboard::Clipboard;
use slint::{ComponentHandle, Model, SharedString, VecModel};

use crate::connect_row_selection::checker::change_number_of_enabled_items;
use crate::{ActiveTab, Callabler, GuiState, MainWindow};

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
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(path) {
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
    
    let mut checked_count_change = 0i64;
    let row_count = model.row_count();
    
    for i in 0..row_count {
        if let Some(mut row) = model.row_data(i) {
            if row.header_row {
                continue;
            }
            
            let path = row.val_str.iter().nth(path_idx).unwrap_or_default();
            
            let is_inside = path.starts_with(filter_path);
            
            let should_check = if select_inside {
                is_inside
            } else {
                !is_inside
            };
            
            if should_check && !row.checked {
                row.checked = true;
                model.set_row_data(i, row);
                checked_count_change += 1;
            }
        }
    }
    
    if checked_count_change > 0 {
        change_number_of_enabled_items(app, active_tab, checked_count_change);
    }
}
