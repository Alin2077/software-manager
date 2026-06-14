#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod registry;
mod files;
mod shortcuts;
mod uninstall;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("软件管理器"),
        ..Default::default()
    };

    eframe::run_native(
        "软件管理器",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(app::SoftwareManagerApp::default()))
        }),
    )
}

/// 加载系统 CJK 字体，解决中文显示为方框/乱码的问题
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 按优先级尝试 Windows 系统内支持中文的字体
    let font_paths = [
        "C:\\Windows\\Fonts\\msyh.ttc",   // 微软雅黑 (TrueType Collection)
        "C:\\Windows\\Fonts\\msyhbd.ttc",  // 微软雅黑 Bold
        "C:\\Windows\\Fonts\\simhei.ttf",  // 黑体
        "C:\\Windows\\Fonts\\simsun.ttc",  // 宋体
        "C:\\Windows\\Fonts\\simkai.ttf",  // 楷体
        "C:\\Windows\\Fonts\\Deng.ttf",    // DengXian (等线)
        "C:\\Windows\\Fonts\\Dengb.ttf",   // DengXian Bold
    ];

    for path in &font_paths {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let font_data = egui::FontData::from_owned(data).tweak(
            egui::FontTweak {
                scale: 0.9,
                ..Default::default()
            },
        );
        fonts.font_data.insert("CJK".to_owned(), std::sync::Arc::new(font_data));

        // 将 CJK 字体插入到 Proportional 和 Monospace 的字体列表最前面
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "CJK".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, "CJK".to_owned());
        }

        ctx.set_fonts(fonts);
        return;
    }

    // 如果所有路径都读不到，仍用默认字体（至少不会崩溃）
    ctx.set_fonts(fonts);
}
