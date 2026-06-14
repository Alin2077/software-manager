use crate::files::{self, FileEntry};
use crate::registry::{self, Software};
use crate::shortcuts::{self, ShortcutInfo};
use crate::uninstall;
use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, TopBottomPanel, Frame};

pub struct SoftwareManagerApp {
    software_list: Vec<Software>,
    selected_index: Option<usize>,
    search_text: String,
    file_tree: Option<FileEntry>,
    shortcuts: Vec<ShortcutInfo>,
    loading: bool,
    detail_loading: bool,
    status_message: String,
}

impl Default for SoftwareManagerApp {
    fn default() -> Self {
        Self {
            software_list: Vec::new(),
            selected_index: None,
            search_text: String::new(),
            file_tree: None,
            shortcuts: Vec::new(),
            loading: true,
            detail_loading: false,
            status_message: "正在加载软件列表...".to_string(),
        }
    }
}

impl SoftwareManagerApp {
    fn load_software(&mut self) {
        self.loading = true;
        self.software_list = registry::get_installed_software();
        self.loading = false;
        self.status_message = format!("已加载 {} 个软件", self.software_list.len());
    }

    fn select_software(&mut self, index: usize) {
        self.selected_index = Some(index);
        let sw = &self.software_list[index];

        self.file_tree = None;
        self.shortcuts.clear();

        if !sw.install_location.is_empty() {
            self.detail_loading = true;
            // 文件树
            self.file_tree = files::scan_directory(&sw.install_location);
            // 快捷方式
            self.shortcuts = shortcuts::find_shortcuts_for_path(&sw.install_location);
            self.detail_loading = false;
        }
    }

    fn run_uninstaller(&self) {
        if let Some(idx) = self.selected_index {
            let cmd = &self.software_list[idx].uninstall_string;
            if !cmd.is_empty() {
                uninstall::run_uninstall_string(cmd);
            }
        }
    }
}

impl eframe::App for SoftwareManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 首次加载
        if self.software_list.is_empty() && self.loading {
            self.load_software();
        }

        // 快捷键：F5 刷新
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.load_software();
            self.selected_index = None;
            self.file_tree = None;
            self.shortcuts.clear();
        }

        // 布局
        TopBottomPanel::top("status_bar")
            .min_height(24.0)
            .frame(Frame::none().fill(Color32::from_rgb(0xF0, 0xF0, 0xF0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_message).size(12.0).color(Color32::GRAY));
                    if self.loading {
                        ui.spinner();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("刷新 (F5)").clicked() {
                            self.load_software();
                            self.selected_index = None;
                            self.file_tree = None;
                            self.shortcuts.clear();
                        }
                    });
                });
            });

        SidePanel::left("software_list")
            .min_width(260.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("已安装软件");
                    ui.add_space(4.0);

                    // 搜索框
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_text)
                            .hint_text("搜索软件名称...")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(4.0);

                    let search_lower = self.search_text.to_lowercase();
                    let filtered_indices: Vec<usize> = self
                        .software_list
                        .iter()
                        .enumerate()
                        .filter(|(_, sw)| {
                            self.search_text.is_empty()
                                || sw.name.to_lowercase().contains(&search_lower)
                        })
                        .map(|(i, _)| i)
                        .collect();

                    let total = self.software_list.len();
                    let filtered_count = filtered_indices.len();
                    let selected_index = self.selected_index;
                    ui.label(format!("{} / {}", filtered_count, total));

                    ui.separator();

                    // 软件列表 — 先克隆需要显示的数据，避免借用冲突
                    let filtered_snapshots: Vec<(usize, Software)> = filtered_indices
                        .iter()
                        .map(|&i| (i, self.software_list[i].clone()))
                        .collect();

                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for (idx, sw) in &filtered_snapshots {
                                let is_selected = selected_index == Some(*idx);
                                let response = ui.selectable_label(
                                    is_selected,
                                    RichText::new(&sw.name).size(13.0),
                                );
                                let mut subtitle = String::new();
                                if !sw.version.is_empty() {
                                    subtitle.push_str(&sw.version);
                                }
                                if !sw.publisher.is_empty() {
                                    if !subtitle.is_empty() {
                                        subtitle.push_str(" · ");
                                    }
                                    subtitle.push_str(&sw.publisher);
                                }
                                if !subtitle.is_empty() {
                                    ui.label(
                                        RichText::new(subtitle)
                                            .size(11.0)
                                            .color(Color32::GRAY),
                                    );
                                }

                                if response.clicked() {
                                    self.select_software(*idx);
                                }
                            }
                        });
                });
            });

        // 中央详情面板
        CentralPanel::default().show(ctx, |ui| {
            if let Some(idx) = self.selected_index {
                let sw = &self.software_list[idx].clone();
                ui.vertical(|ui| {
                    ui.heading(RichText::new(&sw.name).size(18.0));
                    ui.add_space(4.0);

                    // 元信息
                    ui.horizontal(|ui| {
                        if !sw.version.is_empty() {
                            ui.label(format!("版本: {}", sw.version));
                            ui.separator();
                        }
                        if !sw.publisher.is_empty() {
                            ui.label(format!("发行商: {}", sw.publisher));
                            ui.separator();
                        }
                        if !sw.install_location.is_empty() {
                            ui.label(format!("位置: {}", sw.install_location));
                        }
                    });
                    ui.add_space(8.0);

                    // 卸载按钮
                    if !sw.uninstall_string.is_empty() {
                        if ui.button("▶ 打开卸载程序").clicked() {
                            self.run_uninstaller();
                        }
                    }
                    ui.add_space(8.0);

                    ui.separator();

                    // 内容区域
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            if self.detail_loading {
                                ui.spinner();
                                ui.label("正在扫描文件夹...");
                            } else if let Some(ref tree) = self.file_tree {
                                ui.collapsing("📁 安装目录文件", |ui| {
                                    render_file_tree(ui, tree, 0);
                                });
                            } else if !sw.install_location.is_empty() {
                                ui.label("无法读取安装目录");
                            }

                            // 快捷方式
                            if !self.shortcuts.is_empty() {
                                ui.add_space(8.0);
                                ui.collapsing(
                                    format!("🔗 关联快捷方式 ({})", self.shortcuts.len()),
                                    |ui| {
                                        for sc in &self.shortcuts {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("📎").size(12.0));
                                                ui.label(
                                                    RichText::new(&sc.name)
                                                        .size(12.0)
                                                        .strong(),
                                                );
                                                ui.label(
                                                    RichText::new(format!("→ {}", sc.target))
                                                        .size(11.0)
                                                        .color(Color32::GRAY),
                                                );
                                            });
                                        }
                                    },
                                );
                            }
                        });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("← 选择左侧软件查看详情")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                });
            }
        });

        // 每 200ms 请求重绘（对于加载动画）
        if self.loading || self.detail_loading {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

fn render_file_tree(ui: &mut egui::Ui, node: &FileEntry, depth: usize) {
    if node.is_dir {
        let name = format!("📁 {}", node.name);
        let child_count = node.children.len();
        let header = if child_count > 0 {
            format!("{} ({} 项)", name, child_count)
        } else {
            format!("{} (空)", name)
        };

        // egui 的 collapsing 必须给唯一 id
        let id = ui.make_persistent_id(&node.path);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, depth < 2)
            .show_header(ui, |ui| {
                ui.label(RichText::new(header).size(13.0));
            })
            .body(|ui| {
                for child in &node.children {
                    ui.horizontal(|ui| {
                        ui.add_space((depth + 1) as f32 * 16.0);
                        render_file_tree(ui, child, depth + 1);
                    });
                }
            });
    } else {
        let size_str = if node.size > 1024 * 1024 {
            format!("{:.1} MB", node.size as f64 / (1024.0 * 1024.0))
        } else if node.size > 1024 {
            format!("{:.1} KB", node.size as f64 / 1024.0)
        } else {
            format!("{} B", node.size)
        };
        ui.label(
            RichText::new(format!("📄 {}  ", node.name))
                .size(12.0),
        );
        ui.label(
            RichText::new(size_str)
                .size(11.0)
                .color(Color32::GRAY),
        );
    }
}
