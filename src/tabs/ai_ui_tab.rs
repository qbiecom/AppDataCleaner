use crate::ai_config::{AIConfig, AIHandler};
use crate::lang::Language;
use eframe::egui;
use std::sync::{Arc, Mutex};

// 定义标签类型枚举
#[derive(PartialEq)]
enum ConfigTab {
    ApiSettings,
    RetrySettings,
    PromptSettings,
}

pub struct AIConfigurationUI {
    pub show_ai_config_window: bool,
    ai_config: AIConfig,
    ai_handler: Arc<Mutex<AIHandler>>,
    status: Option<String>,
    current_tab: ConfigTab,
    last_config: Option<AIConfig>,
    is_password_visible: bool,
}

impl AIConfigurationUI {
    pub fn new(ai_config: AIConfig, ai_handler: Arc<Mutex<AIHandler>>) -> Self {
        Self {
            show_ai_config_window: false,
            ai_config: ai_config.clone(),
            ai_handler,
            status: None,
            current_tab: ConfigTab::ApiSettings,
            last_config: Some(ai_config),
            is_password_visible: false,
        }
    }

    // 修复未使用变量警告，添加下划线前缀
    pub fn show(&mut self, _ctx: &egui::Context) {
        // 目前此方法为空，但保留参数以便将来可能的用途
    }

    // 添加获取 handler 的方法
    pub fn get_handler(&self) -> Arc<Mutex<AIHandler>> {
        self.ai_handler.clone()
    }

    // 添加获取配置的方法
    pub fn get_config(&self) -> AIConfig {
        self.ai_config.clone()
    }

    // 添加更新状态的方法
    pub fn set_status(&mut self, status: String) {
        self.status = Some(status);
    }

    // 检查配置是否变化并保存
    fn check_and_save_config(&mut self) {
        let config_changed = match &self.last_config {
            Some(last) => {
                // 简单比较一些关键字段
                last.model.url != self.ai_config.model.url
                    || last.model.api_key != self.ai_config.model.api_key
                    || last.model.model != self.ai_config.model.model
                    || last.retry.attempts != self.ai_config.retry.attempts
                    || last.retry.delay != self.ai_config.retry.delay
                    || last.model.prompt != self.ai_config.model.prompt
            }
            None => true,
        };

        if config_changed {
            if let Ok(config_path) = AIConfig::get_config_path() {
                match self.ai_config.save_to_file(config_path.to_str().unwrap()) {
                    Ok(_) => {
                        self.status = Some("配置已自动保存".to_string());
                        // 更新 last_config
                        self.last_config = Some(self.ai_config.clone());
                        // 重新加载配置到 handler
                        if let Ok(mut handler) = self.ai_handler.lock() {
                            handler.update_config(self.ai_config.clone());
                        }
                    }
                    Err(e) => self.status = Some(format!("自动保存失败: {}", e)),
                }
            }
        }
    }

    // 修改为公共方法，可以直接在主面板上调用
    pub fn draw_config_ui(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        // 创建整体垂直布局
        ui.vertical(|ui| {
            // 顶部标签栏
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.current_tab == ConfigTab::ApiSettings, if is_zh { "API设置" } else { "API Settings" })
                    .clicked()
                {
                    self.current_tab = ConfigTab::ApiSettings;
                }
                if ui
                    .selectable_label(self.current_tab == ConfigTab::RetrySettings, if is_zh { "重试设置" } else { "Retry Settings" })
                    .clicked()
                {
                    self.current_tab = ConfigTab::RetrySettings;
                }
                if ui
                    .selectable_label(self.current_tab == ConfigTab::PromptSettings, if is_zh { "Prompt设置" } else { "Prompt Settings" })
                    .clicked()
                {
                    self.current_tab = ConfigTab::PromptSettings;
                }
            });

            ui.separator();

            // 中间部分：根据选中的标签显示对应的内容区域
            ui.group(|ui| match self.current_tab {
                ConfigTab::ApiSettings => self.draw_basic_settings(ui, language),
                ConfigTab::RetrySettings => self.draw_retry_settings(ui, language),
                ConfigTab::PromptSettings => self.draw_prompt_settings(ui, language),
            });

            ui.separator();

            // 显示状态信息
            if let Some(status) = &self.status {
                ui.label(status);
            }

            // 检查配置是否变化并自动保存
            self.check_and_save_config();
        });
    }

    // 添加绘制基本设置的方法
    fn draw_basic_settings(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        ui.horizontal(|ui| {
            ui.heading(if is_zh { "API设置" } else { "API Settings" });
            ui.hyperlink_to(
                if is_zh { "如何获取 API 密钥?" } else { "How to get an API key?" },
                "https://github.com/TC999/AppDataCleaner/issues/48#issuecomment-2674567816",
            );
        });

        let mut changed = false;

        // API 配置
        ui.horizontal(|ui| {
            ui.label(if is_zh { "API地址:" } else { "API URL:" });
            if ui
                .add(egui::TextEdit::singleline(&mut self.ai_config.model.url))
                .changed()
            {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label(if is_zh { "API密钥:" } else { "API Key:" });
            if ui
                .add(
                    egui::TextEdit::singleline(&mut self.ai_config.model.api_key)
                        .password(!self.is_password_visible),
                )
                .changed()
            {
                changed = true;
            }
            if ui.button(if is_zh { "显示/隐藏" } else { "Show/Hide" }).clicked() {
                self.is_password_visible = !self.is_password_visible;
            }
        });

        ui.horizontal(|ui| {
            ui.label(if is_zh { "模型名称:" } else { "Model Name:" });
            if ui
                .add(egui::TextEdit::singleline(&mut self.ai_config.model.model))
                .changed()
            {
                changed = true;
            }
        });

        // 添加测试连接按钮 - 修复生命周期问题
        ui.horizontal(|ui| {
            if ui.button(if is_zh { "测试连接" } else { "Test Connection" }).clicked() {
                // 先保存当前配置
                if changed {
                    self.check_and_save_config();
                }

                // 使用通道来传递连接测试结果
                let (tx, rx) = std::sync::mpsc::channel();
                let handler = self.ai_handler.clone();

                // 创建一个背景线程执行测试
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let result = rt.block_on(async {
                        if let Ok(handler) = handler.lock() {
                            match handler.test_connection().await {
                                Ok(message) => Ok(message),
                                Err(e) => Err(if is_zh {
                                    format!("连接失败: {}", e)
                                } else {
                                    format!("Connection failed: {}", e)
                                }),
                            }
                        } else {
                            Err(if is_zh {
                                "无法获取 AI 处理器锁".to_string()
                            } else {
                                "Cannot acquire AI handler lock".to_string()
                            })
                        }
                    });

                    // 发送结果到主线程
                    let _ = tx.send(result);
                });

                // 尝试立即接收结果（非阻塞）
                match rx.try_recv() {
                    Ok(Ok(success_msg)) => {
                        self.status = Some(success_msg);
                    }
                    Ok(Err(error_msg)) => {
                        self.status = Some(error_msg);
                    }
                    Err(_) => {
                        // 如果没有立即接收到结果，设置一个等待状态
                        self.status = Some(if is_zh {
                            "正在测试连接...".to_string()
                        } else {
                            "Testing connection...".to_string()
                        });

                        // 保存接收器供后续 UI 更新使用
                        let ctx = ui.ctx().clone();
                        std::thread::spawn(move || {
                            // 等待结果
                            if let Ok(result) = rx.recv() {
                                match result {
                                    Ok(msg) => {
                                        // 触发 UI 更新
                                        ctx.request_repaint();
                                        // 将结果通过上下文传递到下一帧 - 修复 Id 类型
                                        ctx.data_mut(|data| {
                                            data.insert_temp("ai_test_result".into(), msg);
                                        });
                                    }
                                    Err(err) => {
                                        ctx.request_repaint();
                                        ctx.data_mut(|data| {
                                            data.insert_temp("ai_test_result".into(), err);
                                        });
                                    }
                                }
                            }
                        });
                    }
                }
            }
        });

        // 检查是否有测试结果更新 - 修复 Id 类型
        ui.ctx().data_mut(|data| {
            if let Some(result) = data.get_temp::<String>("ai_test_result".into()) {
                self.status = Some(result.clone());
                // 使用后移除临时数据
                data.remove::<String>("ai_test_result".into());
            }
        });
    }

    // 添加绘制重试设置的方法
    fn draw_retry_settings(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        ui.heading(if is_zh { "重试设置" } else { "Retry Settings" });
        ui.horizontal(|ui| {
            ui.label(if is_zh { "重试次数:" } else { "Retry attempts:" });
            ui.add(egui::DragValue::new(&mut self.ai_config.retry.attempts).range(1..=10));
        });
        ui.horizontal(|ui| {
            ui.label(if is_zh { "重试延迟(秒):" } else { "Retry delay (s):" });
            ui.add(egui::DragValue::new(&mut self.ai_config.retry.delay).range(1..=60));
        });
    }

    // 添加绘制 Prompt 设置的方法
    fn draw_prompt_settings(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        // 将标题和重置按钮放在同一行
        ui.horizontal(|ui| {
            ui.heading(if is_zh { "Prompt设置" } else { "Prompt Settings" });
            // 添加一些间距，使布局更美观
            ui.add_space(8.0);
            if ui.button(if is_zh { "重置为默认" } else { "Reset to default" }).clicked() {
                // 创建默认配置以获取默认Prompt
                let default_config = AIConfig::default();
                self.ai_config.model.prompt = default_config.model.prompt.clone();

                // 标记为已更改，触发自动保存
                self.status = Some(if is_zh {
                    "已重置为默认Prompt".to_string()
                } else {
                    "Prompt reset to default".to_string()
                });
                self.check_and_save_config();
            }
        });

        // 使用垂直滚动区域来包裹多行文本编辑器
        egui::ScrollArea::vertical()
            .max_height(400.0) // 限制最大高度，确保在小屏幕上也能看到操作状态
            .show(ui, |ui| {
                // 添加多行文本编辑器，占据可用宽度
                let text_edit = egui::TextEdit::multiline(&mut self.ai_config.model.prompt)
                    .desired_width(ui.available_width()) // 占据全部可用宽度
                    .desired_rows(20) // 默认显示20行
                    .code_editor() // 使用代码编辑器风格，更适合编辑提示词
                    .font(egui::TextStyle::Monospace); // 使用等宽字体

                if ui.add(text_edit).changed() {
                    // Prompt 内容发生变化时，自动保存配置
                    self.check_and_save_config();
                }
            });

        // 在底部添加一个提示，告诉用户内容会自动保存
        ui.separator();
        ui.small(if is_zh {
            "提示: 内容修改后会自动保存"
        } else {
            "Tip: Changes are automatically saved"
        });
    }
}
