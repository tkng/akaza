use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Label, Notebook};
use gtk4 as gtk;
use gtk4::gio::ApplicationFlags;
use gtk4::{
    ComboBoxText, FileChooserAction, FileChooserDialog, Grid, ResponseType, ScrolledWindow, Window,
};
use log::{error, info};

use libakaza::config::{Config, DictConfig, DictEncoding, DictType, DictUsage, EngineConfig};

pub fn open_configuration_window() -> Result<()> {
    let config = Arc::new(Mutex::new(Config::load()?));
    let app = Application::new(Some("com.github.akaza.config"), ApplicationFlags::empty());

    app.connect_activate(move |app| {
        connect_activate(app, config.clone()).unwrap();
    });

    let v: Vec<String> = Vec::new();
    app.run_with_args(v.as_slice());
    Ok(())
}

fn connect_activate(app: &Application, config: Arc<Mutex<Config>>) -> Result<()> {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(520)
        .default_height(500)
        .title("Akaza の設定")
        .build();

    let notebook = Notebook::builder().vexpand(true).hexpand(true).build();
    notebook.append_page(
        &build_core_pane(config.clone())?,
        Some(&Label::new(Some("基本設定"))),
    );
    notebook.append_page(
        &build_dict_pane(config.clone())?,
        Some(&Label::new(Some("辞書"))),
    );
    notebook.append_page(&build_about_pane(), Some(&Label::new(Some("アバウト"))));

    let grid = Grid::builder().build();

    grid.attach(&notebook, 0, 0, 6, 1);

    let ok_button = Button::with_label("OK");
    let config = config;
    ok_button.connect_clicked(move |_| {
        eprintln!("Save the configuration...");
        // TODO: 保存処理
        let config = config.lock().unwrap();
        let config = Config {
            keymap: config.keymap.to_string(),
            romkan: config.romkan.to_string(),
            engine: EngineConfig {
                model: config.engine.model.to_string(),
                dicts: config.engine.dicts.clone(),
                dict_cache: true,
            },
        };
        info!("Saving config: {}", serde_yaml::to_string(&config).unwrap());

        config.save().unwrap();

        // 最後に ibus restart をしちゃおう。設定の再読み込みとか実装するのは大変。
        let output = Command::new("ibus").arg("restart").output().unwrap();

        if !output.status.success() {
            error!(
                "Cannot run `ibus restart`: out={}, err={}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        } else {
            info!(
                "Ran `ibus restart`: out={}, err={}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        }
    });
    let cancel_button = Button::with_label("Cancel");
    {
        let window_clone = window.clone();
        cancel_button.connect_clicked(move |_| {
            eprintln!("Close the configuration window!");
            window_clone.close();
        });
    }
    grid.attach(&ok_button, 4, 1, 1, 1);
    grid.attach(&cancel_button, 5, 1, 1, 1);

    window.set_child(Some(&grid));

    window.connect_close_request(move |window| {
        if let Some(application) = window.application() {
            application.remove_window(window);
        }
        Inhibit(false)
    });

    window.show();
    Ok(())
}

#[derive(Debug)]
struct PathConfItem {
    name: String,
    path: String,
}

fn get_list<P>(path: &str, filter: P) -> Vec<PathConfItem>
where
    P: FnMut(&&PathBuf) -> bool,
{
    let p = xdg::BaseDirectories::with_prefix("akaza")
        .unwrap()
        .list_data_files(path);

    p.iter()
        .filter(filter)
        .map(|f| PathConfItem {
            name: f
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .replace(".yml", ""),
            path: f.to_string_lossy().to_string(),
        })
        .collect::<Vec<_>>()
}

fn build_core_pane(config: Arc<Mutex<Config>>) -> Result<Grid> {
    // キーマップとローマ字テーブル、モデルの設定ができるようにする。
    let grid = Grid::new();
    // xalign: 0 は左寄という意味。
    grid.attach(
        &Label::builder().label("キーマップ").xalign(0_f32).build(),
        0,
        0,
        1,
        1,
    );
    grid.attach(
        &{
            let cbt = ComboBoxText::new();
            let keymap = get_list("keymap", |f| f.to_string_lossy().ends_with(".yml"));
            for item in keymap {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt.set_active_id(Some(&config.lock().unwrap().keymap));
            {
                let config = config.clone();
                cbt.connect_changed(move |f| {
                    if let Some(id) = f.active_id() {
                        config.lock().unwrap().keymap = id.to_string();
                    }
                });
            }
            cbt
        },
        1,
        0,
        1,
        1,
    );
    grid.attach(
        &Label::builder()
            .label("ローマ字テーブル")
            .xalign(0_f32)
            .build(),
        0,
        1,
        1,
        1,
    );
    grid.attach(
        &{
            let cbt = ComboBoxText::new();
            let romkan = get_list("romkan", |f| f.to_string_lossy().ends_with(".yml"));
            info!("romkan: {:?}", romkan);
            for item in romkan {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt.set_active_id(Some(&config.lock().unwrap().romkan));

            let config = config.clone();
            cbt.connect_changed(move |f| {
                if let Some(id) = f.active_id() {
                    config.lock().unwrap().romkan = id.to_string();
                }
            });

            cbt
        },
        1,
        1,
        1,
        1,
    );
    grid.attach(
        &Label::builder().label("言語モデル").xalign(0_f32).build(),
        0,
        2,
        1,
        1,
    );
    grid.attach(
        &{
            let cbt = ComboBoxText::new();
            let model = get_list("model", {
                |f| !f.file_name().unwrap().to_string_lossy().starts_with('.')
            });
            info!("model: {:?}", model);
            for item in model {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt.set_active_id(Some(&config.lock().unwrap().engine.model));

            cbt.connect_changed(move |f| {
                if let Some(id) = f.active_id() {
                    config.lock().unwrap().engine.model = id.to_string();
                }
            });

            cbt
        },
        1,
        2,
        1,
        1,
    );
    Ok(grid)
}

fn build_dict_pane(config: Arc<Mutex<Config>>) -> Result<ScrolledWindow> {
    // TODO ここは TreeView 使った方がすっきり書けるはずだが、僕の GTK+ 力が引くすぎて対応できていない。
    // 誰かすっきり使い易くしてほしい。
    fn add_row(grid: &Grid, dict_config: &DictConfig, config: &Arc<Mutex<Config>>, i: usize) {
        grid.attach(
            &Label::builder()
                .xalign(0_f32)
                .label(dict_config.path.as_str())
                .build(),
            0,
            i as i32,
            1,
            1,
        );

        {
            let cbt = ComboBoxText::builder().build();
            for usage in vec![
                DictUsage::Normal,
                DictUsage::SingleTerm,
                DictUsage::Disabled,
            ] {
                cbt.append(Some(usage.as_str()), usage.text_jp());
            }
            cbt.set_active_id(Some(dict_config.usage.as_str()));
            {
                let config = config.clone();
                let path = dict_config.path.clone();
                cbt.connect_changed(move |f| {
                    if let Some(id) = f.active_id() {
                        let mut config = config.lock().unwrap();
                        for mut dict in &mut config.engine.dicts {
                            if dict.path == path {
                                dict.usage = DictUsage::from(&id).unwrap();
                                return;
                            }
                        }
                        config.engine.dicts.push(DictConfig {
                            dict_type: DictType::SKK,
                            encoding: DictEncoding::EucJp,
                            path: path.to_string(),
                            usage: DictUsage::from(&id).unwrap(),
                        })
                    }
                });
            }
            grid.attach(&cbt, 1, i as i32, 1, 1);
        }

        grid.attach(
            &Label::new(Some(dict_config.dict_type.as_str())),
            2,
            i as i32,
            1,
            1,
        );
        {
            let cbt = ComboBoxText::builder().build();
            for encoding in vec![DictEncoding::EucJp, DictEncoding::Utf8] {
                cbt.append(
                    Some(&encoding.to_string()),
                    encoding.as_str().replace('_', "-").as_str(),
                );
            }
            cbt.set_active_id(Some(dict_config.encoding.as_str()));
            {
                let config = config.clone();
                let path = dict_config.path.clone();
                cbt.connect_changed(move |f| {
                    if let Some(id) = f.active_id() {
                        let mut config = config.lock().unwrap();
                        for mut dict in &mut config.engine.dicts {
                            if dict.path == path {
                                dict.encoding = DictEncoding::from(&id).unwrap();
                                break;
                            }
                        }
                    }
                });
            }
            grid.attach(&cbt, 3, i as i32, 1, 1);
        }

        {
            let delete_btn = {
                let path = dict_config.path.clone();
                let config = config.clone();
                let delete_btn = Button::with_label("削除");
                let grid = grid.clone();
                delete_btn.connect_clicked(move |_| {
                    let mut config = config.lock().unwrap();
                    for (i, dict) in &mut config.engine.dicts.iter().enumerate() {
                        if dict.path == path {
                            config.engine.dicts.remove(i);
                            grid.remove_row(i as i32);
                            break;
                        }
                    }
                });
                delete_btn
            };
            grid.attach(&delete_btn, 4, i as i32, 1, 1);
        }
    }

    let scroll = ScrolledWindow::new();

    let parent_grid = Grid::builder().column_spacing(10).build();
    let grid = Grid::builder().column_spacing(10).build();

    let dicts = config.lock().unwrap().engine.dicts.clone();

    for (i, dict_config) in dicts.iter().enumerate() {
        add_row(&grid, dict_config, &config.clone(), i);
    }

    parent_grid.attach(&grid, 0, 0, 1, 1);

    {
        let add_btn = {
            let add_btn = Button::with_label("Add");
            let config = config;
            let grid = grid;
            add_btn.connect_clicked(move |_| {
                let dialog = FileChooserDialog::new(
                    Some("辞書の選択"),
                    None::<&Window>,
                    FileChooserAction::Open,
                    &[
                        ("開く", ResponseType::Accept),
                        ("キャンセル", ResponseType::None),
                    ],
                );
                let config = config.clone();
                let grid = grid.clone();
                dialog.connect_response(move |dialog, resp| match resp {
                    ResponseType::Accept => {
                        let file = dialog.file().unwrap();
                        let path = file.path().unwrap();

                        info!("File: {:?}", path);
                        let dict_config = &DictConfig {
                            path: path.to_string_lossy().to_string(),
                            encoding: DictEncoding::Utf8,
                            usage: DictUsage::Normal,
                            dict_type: DictType::SKK,
                        };
                        config
                            .lock()
                            .unwrap()
                            .engine
                            .dicts
                            .push(dict_config.clone());
                        add_row(
                            &grid,
                            dict_config,
                            &config.clone(),
                            config.lock().unwrap().engine.dicts.len(),
                        );
                        dialog.close();
                    }
                    ResponseType::Close
                    | ResponseType::Reject
                    | ResponseType::Yes
                    | ResponseType::No
                    | ResponseType::None
                    | ResponseType::DeleteEvent => {
                        dialog.close();
                    }
                    _ => {}
                });
                dialog.show();
            });
            add_btn
        };
        parent_grid.attach(&add_btn, 0, 1, 1, 1);
    }
    scroll.set_child(Some(&parent_grid));
    Ok(scroll)
}

fn build_about_pane() -> Label {
    Label::new(Some(
        format!("Akaza version {}", env!("CARGO_PKG_VERSION")).as_str(),
    ))
}
