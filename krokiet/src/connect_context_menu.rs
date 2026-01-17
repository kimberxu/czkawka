use std::path::Path;
use slint::{ComponentHandle, Model};
use crate::{Callabler, GuiState, MainWindow};
// use crate::ActiveTab;

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
