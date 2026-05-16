use crate::logger;
use crate::ai_config::{AIConfig, AIHandler};
use crate::lang::Language;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use crate::tabs::about_tab;
use crate::tabs::ai_ui_tab::AIConfigurationUI;
use crate::tabs::clear_tab::ClearTabState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TopTab {
    Home,
    About,
    AIConfig,
}

pub struct AppDataCleaner {
    // 标签页状态
    current_tab: TopTab,             // 当前选中的标签页
    show_about_window: bool,

    // 日志相关字段
    is_logging_enabled: bool,
    previous_logging_state: bool,

    // 主题相关字段
    dark_mode: bool,                 // 深色模式开关
    language: Language,

    // 清理标签页状态
    clear_tab: ClearTabState,

    // AI UI标签页
    ai_ui: AIConfigurationUI,
    ai_rx: Option<Receiver<(String, String, String)>>, // 添加 AI 响应接收器
}

impl Default for AppDataCleaner {
    fn default() -> Self {
        let (ai_tx, ai_rx) = std::sync::mpsc::channel();  // 创建 AI 通信通道
        
        // 加载AI配置
        let ai_config = match AIConfig::load_from_file("folders_description.yaml") {
            Ok(config) => {
                logger::log_info("已成功加载AI配置文件");
                config
            }
            Err(_) => {
                logger::log_info("未找到配置文件，使用默认配置");
                AIConfig::default()
            }
        };

        // 创建 AIHandler 并包装在 Arc<Mutex<>> 中
        let ai_handler = Arc::new(Mutex::new(AIHandler::new(
            ai_config.clone(),
            Some(ai_tx.clone())
        )));

        let ai_ui = AIConfigurationUI::new(ai_config.clone(), ai_handler.clone());

        // 创建清理标签页状态
        let mut clear_tab = ClearTabState::default();
        
        // 设置回调函数 - 使用 String 而不是引用
        {
            let ai_handler_clone = ai_handler.clone();
            clear_tab.set_generate_description_callback(move |folder| {
                let folder_name = folder.to_string();
                // 这里使用了副本，不再引用原始对象
                let selected_folder_clone = "Roaming".to_string(); // 默认值，将在实际使用时更新
                let handler = ai_handler_clone.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Ok(mut handler) = handler.lock() {
                            if let Err(e) = handler.generate_single_description(folder_name.clone(), selected_folder_clone).await {
                                logger::log_error(&format!("生成描述失败: {}", e));
                            }
                        }
                    });
                });
            });
        }

        // 设置批量生成描述回调 - 不再从clear_tab捕获变量
        {
            let ai_handler_clone = ai_handler.clone();
            clear_tab.set_generate_all_descriptions_callback(move |folder_data, selected_folder| {
                let folder_data = folder_data.clone();
                let selected_folder = selected_folder.to_string();
                let handler = ai_handler_clone.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Ok(mut handler) = handler.lock() {
                            if let Err(e) = handler.generate_all_descriptions(folder_data.clone(), selected_folder).await {
                                logger::log_error(&format!("批量生成描述失败: {}", e));
                            }
                        }
                    });
                });
            });
        }

        Self {
            // 界面状态初始化
            show_about_window: false,
            current_tab: TopTab::Home,  // 默认选中主页标签

            // 日志相关初始化
            is_logging_enabled: false,
            previous_logging_state: false,

            // 主题相关初始化
            dark_mode: true,  // 默认使用深色模式
            language: Language::Chinese,

            // 清理标签页初始化 
            clear_tab,

            // AI相关初始化
            ai_ui,
            ai_rx: Some(ai_rx),  // 保存 AI 响应接收器
        }
    }
}

impl AppDataCleaner {
    fn setup_custom_fonts(&self, ctx: &egui::Context) {
        use eframe::egui::{FontData, FontDefinitions};

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "custom_font".to_owned(),
            FontData::from_static(include_bytes!("../assets/SourceHanSansCN-Regular.otf")),
        );

        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["custom_font".to_owned()],
        );
        fonts
            .families
            .insert(egui::FontFamily::Monospace, vec!["custom_font".to_owned()]);

        ctx.set_fonts(fonts);
    }

    fn show_top_menu(&mut self, ctx: &egui::Context) {
        let is_zh = self.language.is_chinese();
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {  
                // 左侧标签页和选项
                ui.selectable_value(&mut self.current_tab, TopTab::Home, if is_zh { "主页" } else { "Home" });
            ui.selectable_value(&mut self.current_tab, TopTab::About, if is_zh { "关于" } else { "About" });
            ui.selectable_value(&mut self.current_tab, TopTab::AIConfig, if is_zh { "AI配置" } else { "AI Config" });
            ui.label("|"); // 添加分隔符
            if ui.button(if is_zh { "清理 Temp" } else { "Clean Temp" }).clicked() {
                self.clear_tab.clean_temp_directory();
            }
            ui.label("|"); // 添加分隔符
                ui.checkbox(&mut self.is_logging_enabled, if is_zh { "启用日志" } else { "Enable Logging" });

                // 添加一个弹性空间，将后面的内容推到右侧
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let language_text = if is_zh { "English" } else { "中文" };
                    if ui.button(language_text).clicked() {
                        self.language.toggle();
                    }

                    ui.separator();

                    // 切换文件夹按钮
                    ui.menu_button(if is_zh { "切换文件夹" } else { "Switch Folder" }, |ui| {
                        for folder in ["Roaming", "Local", "LocalLow"] {
                            if ui.button(folder).clicked() {
                                self.clear_tab.set_selected_appdata_folder(folder.to_string());
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button(if is_zh { "选择自定义文件夹..." } else { "Select custom folder..." }).clicked() {
                            self.clear_tab.open_custom_folder_dialog();
                            ui.close_menu();
                        }
                    });
                    // 当前目标文件夹显示
                    let display_name = if let Some(custom_path) = &self.clear_tab.custom_folder_path {
                        if is_zh {
                            format!("自定义: {}", custom_path.display())
                        } else {
                            format!("Custom: {}", custom_path.display())
                        }
                    } else {
                        self.clear_tab.selected_appdata_folder.clone()
                    };
                    ui.label(if is_zh {
                        format!("当前目标: {}", display_name)
                    } else {
                        format!("Current target: {}", display_name)
                    });
                    
                    ui.separator(); // 分隔符
                    
                    // 主题切换按钮
                    let theme_text = if self.dark_mode {
                        if is_zh { "☀ 浅色" } else { "☀ Light" }
                    } else if is_zh {
                        "🌙 深色"
                    } else {
                        "🌙 Dark"
                    };
                    if ui.button(theme_text).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });

            ui.separator();
        });
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_custom_fonts(ctx);
        
        // 设置主题
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        // 处理日志开关
        if self.is_logging_enabled != self.previous_logging_state {
            logger::init_logger(self.is_logging_enabled); // 初始化日志系统
            if self.is_logging_enabled {
                logger::log_info("日志系统已启用");
            } else {
                logger::log_info("日志系统已禁用");
            }
            self.previous_logging_state = self.is_logging_enabled; // 更新状态
        }
        
        // 处理 AI 响应，忽略不需要的变量
        if let Some(rx) = &self.ai_rx {
            while let Ok((_, folder_name, _)) = rx.try_recv() {
                // 重新加载描述文件以更新显示
                self.clear_tab.update_folder_descriptions();
                // 更新状态
                self.clear_tab.status = Some(if self.language.is_chinese() {
                    format!("已更新 {} 的描述", folder_name)
                } else {
                    format!("Updated description for {}", folder_name)
                });
                // 强制重绘
                ctx.request_repaint();
            }
        }

        // 顶部菜单
        self.show_top_menu(ctx);

        // 主面板 - 根据当前标签页显示不同内容
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                TopTab::Home => self.clear_tab.show(ui, self.language),
                TopTab::About => about_tab::handle_about_tab(ui, self.language),
                TopTab::AIConfig => self.ai_ui.draw_config_ui(ui, self.language),
            }
        });

        // 窗口显示
        self.show_windows(ctx);
    }

    fn show_windows(&mut self, ctx: &egui::Context) {
        // 关于窗口
        if self.show_about_window {
            about_tab::show_about_window(ctx, &mut self.show_about_window, self.language);
        }

        // AI配置窗口(使用重构后的AI UI模块)
        self.ai_ui.show(ctx);

        // 移动窗口
        self.clear_tab.move_module.show_move_window(ctx, self.language);
    }
}

impl eframe::App for AppDataCleaner {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update(ctx, frame);
    }
}
