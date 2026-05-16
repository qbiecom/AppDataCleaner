use eframe::egui;
use crate::lang::Language;

// 窗口形式显示关于内容
pub fn show_about_window(ctx: &egui::Context, open: &mut bool, language: Language) {
    egui::Window::new(if language.is_chinese() { "关于此软件" } else { "About" })
        .open(open)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            show_about_content(ui, language);
        });
}

// 面板形式显示关于内容
pub fn show_about_content(ui: &mut egui::Ui, language: Language) {
    let is_zh = language.is_chinese();
    let version = env!("CARGO_PKG_VERSION");

    ui.heading("AppData Cleaner");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label(if is_zh { "作者: " } else { "Author: " });
        ui.hyperlink_to("TC999", "https://github.com/TC999");
    });

    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label(if is_zh { "源代码仓库:" } else { "Source repository:" });
        ui.hyperlink("https://github.com/TC999/AppDataCleaner");
    });

    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label(if is_zh { "议题反馈:" } else { "Issue feedback:" });
        ui.hyperlink_to("GitHub Issues", "https://github.com/TC999/AppDataCleaner/issues");
    });

    ui.add_space(10.0);

    ui.label(if is_zh {
        format!("版本: {}", version)
    } else {
        format!("Version: {}", version)
    });
    ui.label(if is_zh { "许可证: GPL-3.0" } else { "License: GPL-3.0" });

    ui.add_space(20.0);

    ui.heading(if is_zh { "鸣谢:" } else { "Acknowledgments:" });
    ui.label(if is_zh {
        "egui - 一个简单、快速、高度可移植的即时模式 GUI 库"
    } else {
        "egui - a simple, fast and highly portable immediate-mode GUI library"
    });
    ui.hyperlink_to(if is_zh { "egui 官方网站" } else { "egui official site" }, "https://github.com/emilk/egui");

    ui.add_space(10.0);
    ui.heading(if is_zh { "贡献者:" } else { "Contributors:" });
    ui.hyperlink_to("Xch13", "https://github.com/Xch13");
}

// 处理关于标签页面
pub fn handle_about_tab(ui: &mut egui::Ui, language: Language) {
    show_about_content(ui, language);
}
