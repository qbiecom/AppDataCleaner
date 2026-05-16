use crate::confirmation::show_confirmation;
use crate::database::{Database, get_default_db_path, database_exists};
use crate::lang::Language;
use crate::stats::Stats;
use crate::stats_logger::StatsLogger;
use crate::yaml_loader::{load_folder_descriptions, FolderDescriptions};
use crate::{confirmation, delete, ignore, logger, move_module, open, scanner, utils};
use eframe::egui::{self, Grid, ScrollArea};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::fs;
use std::thread; // 引入 StatsLogger 模块

pub struct ClearTabState {
    // 基础字段
    pub is_scanning: bool,
    pub folder_data: Vec<(String, u64)>,
    pub selected_appdata_folder: String,
    pub custom_folder_path: Option<PathBuf>, // 新增：自定义文件夹路径
    pub tx: Option<Sender<(String, u64)>>,
    pub rx: Option<Receiver<(String, u64)>>,
    pub total_size: u64,

    pub db: Database, // 新增字段

    // 界面状态字段
    pub confirm_delete: Option<(String, bool)>,
    pub status: Option<String>,

    // 排序相关字段
    pub sort_criterion: Option<String>, // 排序标准:"name"或"size"
    pub sort_order: Option<String>,     // 排序顺序:"asc"或"desc"

    // 文件夹描述相关
    pub folder_descriptions: Option<FolderDescriptions>,
    pub yaml_error_logged: bool,
    pub ignored_folders: HashSet<String>,

    // 移动模块
    pub move_module: move_module::MoveModule,

    // 生成描述的回调函数
    generate_description_callback: Option<Box<dyn Fn(&str) + Send>>,
    generate_all_descriptions_callback: Option<Box<dyn Fn(&Vec<(String, u64)>, &str) + Send>>,

    // 多选操作
    pub selected_folders: HashSet<String>, // 新增字段，存储选中的文件夹

    // 新增字段
    pub stats: Stats,
    pub stats_logger: StatsLogger, // 新增字段
    pub is_cleaning_temp: bool, // 是否正在清理Temp目录

    // 删除数据库确认相关字段
    pub show_delete_db_confirmation: bool,
}

impl Default for ClearTabState {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let db_path = get_default_db_path();
        let db = Database::new(&db_path).unwrap_or_else(|_| panic!("无法初始化数据库: {}", db_path));
        Self {
            db,
            // 基础字段初始化
            is_scanning: false,
            folder_data: vec![],
            selected_appdata_folder: "Roaming".to_string(),
            custom_folder_path: None, // 初始化为None
            tx: Some(tx),
            rx: Some(rx),
            total_size: 0,

            // 界面状态初始化
            confirm_delete: None,
            status: Some("未扫描".to_string()),

            // 排序相关初始化
            sort_criterion: None,
            sort_order: None,

            // 文件夹描述相关初始化
            folder_descriptions: None,
            yaml_error_logged: false,
            ignored_folders: ignore::load_ignored_folders(),

            // 移动模块初始化
            move_module: Default::default(),

            // 回调函数初始化为 None
            generate_description_callback: None,
            generate_all_descriptions_callback: None,

            // 多选操作初始化
            selected_folders: HashSet::new(), // 初始化为空集合

            // 新增字段初始化
            stats: Stats::new(),
            stats_logger: StatsLogger::new(PathBuf::from("stats.log")), // 初始化 StatsLogger
            is_cleaning_temp: false, // 初始化为false

            // 删除数据库确认相关字段初始化
            show_delete_db_confirmation: false,
        }
    }
}

// 其他代码保持不变

impl ClearTabState {
    // 新增：实现 handle_folder_operations 方法
    fn handle_folder_operations(&mut self, ui: &mut egui::Ui, folder: &str, size: u64, language: Language) {
        // 显示复选框，用于多选操作
        let mut is_selected = self.selected_folders.contains(folder);
        if ui.checkbox(&mut is_selected, "").clicked() {
            if is_selected {
                self.selected_folders.insert(folder.to_string());
            } else {
                self.selected_folders.remove(folder);
            }
        }

        // 显示文件夹名称和大小
        if self.ignored_folders.contains(folder) {
            ui.add_enabled(
                false,
                egui::Label::new(egui::RichText::new(folder).color(egui::Color32::GRAY)),
            );
        } else {
            ui.label(folder);
        }
        ui.label(utils::format_size(size));

        // 显示描述
        self.show_folder_description(ui, folder, language);

        // 显示操作按钮
        self.show_folder_actions(ui, folder, language);
    }
    pub fn new() -> Self {
        let db_path = get_default_db_path();
        let db = Database::new(&db_path).unwrap_or_else(|_| panic!("无法初始化数据库: {}", db_path));
        Self {
            db,
            ..Default::default()
        }
    }

    pub fn set_generate_description_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + 'static,
    {
        self.generate_description_callback = Some(Box::new(callback));
    }

    pub fn set_generate_all_descriptions_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Vec<(String, u64)>, &str) + Send + 'static,
    {
        self.generate_all_descriptions_callback = Some(Box::new(callback));
    }

    // 抽取文件夹操作逻辑到单独的方法
    pub fn handle_delete_confirmation(
        ctx: &egui::Context,
        confirm_delete: &mut Option<(String, bool)>,
        selected_appdata_folder: &str,
        status: &mut Option<String>,
        folder_data: &mut Vec<(String, u64)>, // 新增参数
        stats: &mut Stats,                    // 新增参数
        stats_logger: &StatsLogger,           // 新增参数
        db: &Database,                        // 新增参数
        language: Language,
    ) {
        let is_zh = language.is_chinese();
        if let Some((folder_name, is_bulk)) = confirm_delete.clone() {
            if is_bulk && folder_name == "BULK_DELETE" {
                let message = if is_zh {
                    "确定要批量删除选中的文件夹吗？"
                } else {
                    "Are you sure you want to batch delete selected folders?"
                };
                if let Some(confirm) = show_confirmation(ctx, message, status, language) {
                    if confirm {
                        let selected_folders: Vec<String> = folder_data
                            .iter()
                            .filter(|(_folder, _)| confirm_delete.as_ref().map_or(false, |c| c.1))
                            .map(|(folder, _)| folder.clone())
                            .collect();

                        for folder in &selected_folders {
                            if let Some(base_path) = utils::get_appdata_dir(selected_appdata_folder)
                            {
                                let full_path = base_path.join(&folder);
                                if let Err(err) =
                                    delete::delete_folder(&full_path, stats, stats_logger, db, selected_appdata_folder)
                                {
                                    logger::log_error(&format!("批量删除失败: {}", err));
                                } else {
                                    logger::log_info(&format!("已删除文件夹: {}", folder));
                                }
                            }
                        }
                        folder_data.retain(|(folder, _)| !selected_folders.contains(folder));
                        *status = Some(if is_zh {
                            "批量删除完成".to_string()
                        } else {
                            "Batch deletion completed".to_string()
                        });
                    }
                    *confirm_delete = None;
                }
            } else {
                let message = if is_zh {
                    format!("确定要彻底删除文件夹 {} 吗？", folder_name)
                } else {
                    format!("Are you sure you want to permanently delete folder {}?", folder_name)
                };
                if let Some(confirm) = show_confirmation(ctx, &message, status, language) {
                    if confirm {
                        if let Some(base_path) = utils::get_appdata_dir(selected_appdata_folder) {
                            let full_path = base_path.join(&folder_name);
                            if let Err(err) = delete::delete_folder(&full_path, stats, stats_logger, db, selected_appdata_folder)
                            {
                                logger::log_error(&format!("删除失败: {}", err));
                            } else {
                                logger::log_info(&format!("已删除文件夹: {}", folder_name));
                                folder_data.retain(|(folder, _)| folder != &folder_name);
                            }
                            *status = Some(if is_zh {
                                format!("文件夹 {} 已成功删除", folder_name)
                            } else {
                                format!("Folder {} deleted successfully", folder_name)
                            });
                        }
                    }
                    *confirm_delete = None;
                }
            }
        }
    }

    fn show_folder_description(&self, ui: &mut egui::Ui, folder: &str, language: Language) {
        let description = self
            .folder_descriptions
            .as_ref()
            .and_then(|desc| desc.get_description(folder, &self.selected_appdata_folder));

        match description {
            Some(desc) => ui.label(desc),
            None => ui.label(if language.is_chinese() { "无描述" } else { "No description" }),
        };
    }

    fn show_folder_actions(&mut self, ui: &mut egui::Ui, folder: &str, language: Language) {
        let is_zh = language.is_chinese();
        let is_ignored = self.ignored_folders.contains(folder);

        if !is_ignored {
            if ui.button(if is_zh { "彻底删除" } else { "Delete" }).clicked() {
                self.confirm_delete = Some((folder.to_string(), false));
                self.status = None;
            }
            if ui.button(if is_zh { "移动" } else { "Move" }).clicked() {
                self.move_module.show_window = true;
                self.move_module.folder_name = folder.to_string();
            }
            if ui.button(if is_zh { "忽略" } else { "Ignore" }).clicked() {
                self.ignored_folders.insert(folder.to_string());
                ignore::save_ignored_folders(&self.ignored_folders);
                logger::log_info(&format!("文件夹 '{}' 已被忽略", folder));
            }
        } else {
            ui.add_enabled(false, |ui: &mut egui::Ui| {
                let response1 = ui.button(if is_zh { "彻底删除" } else { "Delete" });
                let response2 = ui.button(if is_zh { "移动" } else { "Move" });
                let response3 = ui.button(if is_zh { "忽略" } else { "Ignore" });
                response1 | response2 | response3
            });
        }

        if ui.button(if is_zh { "打开" } else { "Open" }).clicked() {
            if let Some(base_path) = utils::get_appdata_dir(&self.selected_appdata_folder) {
                let full_path = base_path.join(folder);
                if let Err(err) = open::open_folder(&full_path) {
                    logger::log_error(&format!("无法打开文件夹: {}", err));
                }
            }
        }

        if ui.button(if is_zh { "生成描述" } else { "Generate Description" }).clicked() {
            self.generate_description(folder, language);
        }
    }

    fn generate_description(&mut self, folder: &str, language: Language) {
        if let Some(callback) = &self.generate_description_callback {
            self.status = Some(if language.is_chinese() {
                format!("正在为 {} 生成描述...", folder)
            } else {
                format!("Generating description for {}...", folder)
            });
            // 传递实际的文件夹名和当前选中的AppData文件夹
            callback(folder);
        }
    }

    pub fn show_sort_controls(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        ui.horizontal(|ui| {
            // 添加排序按钮
            ui.menu_button(if is_zh { "排序" } else { "Sort" }, |ui| {
                if ui.button(if is_zh { "名称正序" } else { "Name Asc" }).clicked() {
                    self.sort_criterion = Some("name".to_string());
                    self.sort_order = Some("asc".to_string());
                }         
                if ui.button(if is_zh { "名称倒序" } else { "Name Desc" }).clicked() {
                    self.sort_criterion = Some("name".to_string());
                    self.sort_order = Some("desc".to_string());
                }
                if ui.button(if is_zh { "大小正序" } else { "Size Asc" }).clicked() {
                    self.sort_criterion = Some("size".to_string());
                    self.sort_order = Some("asc".to_string());
                }
                if ui.button(if is_zh { "大小倒序" } else { "Size Desc" }).clicked() {
                    self.sort_criterion = Some("size".to_string());
                    self.sort_order = Some("desc".to_string());
                }
            });
            
            // 数据库状态显示
            self.show_database_status(ui, language);
        });

        // 计算总大小
        self.total_size = self.folder_data.iter().map(|(_, size)| size).sum();

        ui.horizontal(|ui| {
            // 显示总大小
            ui.label(if is_zh {
                format!("总大小: {}", utils::format_size(self.total_size))
            } else {
                format!("Total size: {}", utils::format_size(self.total_size))
            });

            // 显示总清理数和总大小
            ui.label(if is_zh {
                format!("已清理文件夹数量: {}", self.stats.cleaned_folders_count)
            } else {
                format!("Cleaned folder count: {}", self.stats.cleaned_folders_count)
            });
            ui.label(if is_zh {
                format!("总清理大小: {}", utils::format_size(self.stats.total_cleaned_size))
            } else {
                format!("Total cleaned size: {}", utils::format_size(self.stats.total_cleaned_size))
            });
        });
    }

    fn show_database_status(&self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        let db_path = get_default_db_path();
        
        if database_exists(&db_path) {
            ui.label("📊");
            if ui.button(if is_zh { "数据库状态" } else { "Database Status" }).clicked() {
                // 可以在这里添加详细的数据库状态窗口
            }
            
            // 显示数据库统计信息（如果能够打开数据库）
            if let Ok(db) = Database::new(&db_path) {
                if let Ok((total_records, last_updated)) = db.get_stats() {
                    ui.label(if is_zh {
                        format!("记录数: {}", total_records)
                    } else {
                        format!("Records: {}", total_records)
                    });
                    if last_updated != "无数据" {
                        // 只显示日期部分，不显示完整时间戳
                        if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&last_updated) {
                            let date_str = datetime.format("%Y-%m-%d %H:%M").to_string();
                            ui.label(if is_zh {
                                format!("更新: {}", date_str)
                            } else {
                                format!("Updated: {}", date_str)
                            });
                        } else {
                            ui.label(if is_zh { "更新: 最近" } else { "Updated: recent" });
                        }
                    }
                }
            }
        } else {
            ui.label(if is_zh {
                "🔍 首次扫描将创建数据库"
            } else {
                "🔍 First scan will create database"
            });
        }
    }

    pub fn show_folder_grid(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        Grid::new("folders_table").striped(true).show(ui, |ui| {
            ui.label(if is_zh { "文件夹" } else { "Folder" });
            ui.label(if is_zh { "大小" } else { "Size" });
            ui.label(if is_zh { "描述" } else { "Description" });
            ui.label(if is_zh { "操作" } else { "Actions" });
            ui.end_row();

            // 先排序
            if let Some(criterion) = &self.sort_criterion {
                self.folder_data.sort_by(|a, b| {
                    if *criterion == "name" {
                        if self.sort_order == Some("asc".to_string()) {
                            a.0.cmp(&b.0)
                        } else {
                            b.0.cmp(&a.0)
                        }
                    } else {
                        if self.sort_order == Some("asc".to_string()) {
                            a.1.cmp(&b.1)
                        } else {
                            b.1.cmp(&a.1)
                        }
                    }
                });
            }

            // 创建一个临时向量来存储需要处理的数据
            let folder_data = self.folder_data.clone();

            // 使用临时数据进行遍历
            for (folder, size) in folder_data {
                self.handle_folder_operations(ui, &folder, size, language);
                ui.end_row();
            }
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        // 初始化if未加载folder descriptions
        if self.folder_descriptions.is_none() {
            self.folder_descriptions =
                load_folder_descriptions("folders_description.yaml", &mut self.yaml_error_logged);
        }

        // 删除确认弹窗逻辑
        confirmation::handle_delete_confirmation(
            ui.ctx(),
            &mut self.confirm_delete,
            &self.selected_appdata_folder,
            &mut self.status,
            &mut self.folder_data,
            &mut self.selected_folders,
            &mut self.stats,
            &self.stats_logger,
            &self.db,
            language,
        );

        // 扫描按钮和生成描述按钮放在一起
        ui.horizontal(|ui| {
            if ui.button(if is_zh { "立即扫描" } else { "Scan Now" }).clicked() && !self.is_scanning {
                self.is_scanning = true;
                self.folder_data.clear();
                self.status = Some(if is_zh { "扫描中...".to_string() } else { "Scanning...".to_string() });

                let tx = self.tx.clone().unwrap();
                let folder_type = self.selected_appdata_folder.clone();

                scanner::scan_appdata(tx, &folder_type);
            }

            // 一键生成所有描述按钮
            if ui.button(if is_zh { "一键生成所有描述" } else { "Generate All Descriptions" }).clicked() {
                if let Some(callback) = &self.generate_all_descriptions_callback {
                    self.status = Some(if is_zh {
                        "正在生成描述...".to_string()
                    } else {
                        "Generating descriptions...".to_string()
                    });
                    callback(&self.folder_data, &self.selected_appdata_folder);
                }
            }

            // 删除数据库按钮
            if ui.button(if is_zh { "删除数据库" } else { "Delete Database" }).clicked() {
                self.show_delete_db_confirmation = true;
            }
        });

        // 添加批量操作按钮
        self.show_bulk_actions(ui, language);

        // 接收扫描结果
        if let Some(rx) = &self.rx {
            while let Ok((folder, size)) = rx.try_recv() {
                // 检查是否接收到扫描完成标志
                if folder == "__SCAN_COMPLETE__" {
                    self.is_scanning = false;
                    self.status = Some(if is_zh {
                        "扫描完成".to_string()
                    } else {
                        "Scan completed".to_string()
                    });
                } else if folder == "__TEMP_CLEANUP_COMPLETE__" {
                    // 处理Temp目录清理完成的标志
                    self.is_cleaning_temp = false;
                } else if folder.starts_with("__STATUS__") {
                    // 处理状态消息
                    let status_msg = folder.strip_prefix("__STATUS__").unwrap_or(&folder);
                    self.status = Some(status_msg.to_string());
                } else {
                    self.folder_data.push((folder, size));
                }
            }
        }

        // 显示状态
        if let Some(status) = &self.status {
            ui.label(status);
        }

        // 排序控件
        self.show_sort_controls(ui, language);

        // 文件夹列表
        ScrollArea::vertical().show(ui, |ui| {
            self.show_folder_grid(ui, language);
        });

        // 删除数据库确认对话框
        if self.show_delete_db_confirmation {
            egui::Window::new(if is_zh { "确认删除数据库" } else { "Confirm Database Deletion" })
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(if is_zh { "确定要删除数据库吗？" } else { "Are you sure you want to delete the database?" });
                    ui.horizontal(|ui| {
                        if ui.button(if is_zh { "是" } else { "Yes" }).clicked() {
                            // 删除数据库文件
                            let db_path = get_default_db_path();
                            let _ = std::fs::remove_file(&db_path);

                            // 更新状态
                            self.status = Some(if is_zh {
                                "数据库已删除".to_string()
                            } else {
                                "Database deleted".to_string()
                            });
                            self.show_delete_db_confirmation = false;
                        }
                        if ui.button(if is_zh { "否" } else { "No" }).clicked() {
                            self.show_delete_db_confirmation = false;
                        }
                    });
                });
        }
    }

    pub fn show_bulk_actions(&mut self, ui: &mut egui::Ui, language: Language) {
        let is_zh = language.is_chinese();
        ui.horizontal(|ui| {
            if ui.button(if is_zh { "批量删除" } else { "Batch Delete" }).clicked() {
                for folder in &self.selected_folders {
                    if self.ignored_folders.contains(folder) {
                        self.status = Some(if is_zh {
                            format!("文件夹 '{}' 在忽略名单中，无法删除", folder)
                        } else {
                            format!("Folder '{}' is in ignore list and cannot be deleted", folder)
                        });
                        logger::log_info(&format!("文件夹 '{}' 在忽略名单中，无法删除", folder));
                        return;
                    }
                }

                if !self.selected_folders.is_empty() {
                    self.confirm_delete = Some(("BULK_DELETE".to_string(), true));
                    self.status = None; // 确保状态信息不影响按钮显示
                } else {
                    self.status = Some(if is_zh {
                        "未选择任何文件夹，无法执行批量删除".to_string()
                    } else {
                        "No folder selected for batch deletion".to_string()
                    });
                }
            }

            if ui.button(if is_zh { "批量忽略" } else { "Batch Ignore" }).clicked() {
                for folder in &self.selected_folders {
                    self.ignored_folders.insert(folder.to_string());
                    logger::log_info(&format!("文件夹 '{}' 已被忽略", folder));
                }
                ignore::save_ignored_folders(&self.ignored_folders);
                self.selected_folders.clear();
            }
        });
    }

    // 设置选中的AppData文件夹
    pub fn set_selected_appdata_folder(&mut self, folder: String) {
        self.custom_folder_path = None; // 清除自定义文件夹路径
        self.selected_appdata_folder = folder.clone();
        self.folder_data.clear();
        self.is_scanning = false;
        self.status = Some("未扫描".to_string());

        // 尝试加载数据库缓存（如果有）
        if let Ok(db) = crate::database::Database::new("appdata_cleaner.db") {
            if db.has_data_for_type(&folder).unwrap_or(false) {
                // 有缓存则直接加载
                if let Ok(records) = db.get_folders_by_type(&folder) {
                    self.folder_data = records.iter().map(|r| (r.folder_name.clone(), r.folder_size)).collect();
                    self.is_scanning = false;
                    self.status = Some("已加载缓存".to_string());
                    return;
                }
            }
        }
        // 没有缓存则自动触发扫描
        self.is_scanning = true;
        self.status = Some("扫描中...".to_string());
        if let Some(tx) = self.tx.clone() {
            let folder_type = self.selected_appdata_folder.clone();
            crate::scanner::scan_appdata(tx, &folder_type);
        }
    }

    // 更新文件夹描述
    pub fn update_folder_descriptions(&mut self) {
        self.folder_descriptions =
            load_folder_descriptions("folders_description.yaml", &mut self.yaml_error_logged);
    }

    // 清理Temp目录的方法
    pub fn clean_temp_directory(&mut self) {
        if self.is_cleaning_temp {
            return; // 已经在清理中，避免重复操作
        }
        
        self.is_cleaning_temp = true;
        self.status = Some("开始清理Temp目录...".to_string());
        
        // 克隆所需的数据用于线程
        
        // 保存tx的引用，用于发送状态消息
        let temp_tx = self.tx.clone();
        
        // 创建清理线程
        thread::spawn(move || {
            let mut total_cleaned_size = 0;
            let mut deleted_files = 0;
            let mut skipped_files = 0;
            
            // 创建日志上下文
            let log_ctx = logger::LogContext::new("TEMP_CLEANUP");
            
            // 获取Temp目录
            if let Some(temp_dir) = crate::utils::get_temp_dir() {
                println!("正在清理Temp目录: {}", temp_dir.display());
                crate::logger::log_info(&format!("正在清理Temp目录: {}", temp_dir.display()));
                
                // 遍历Temp目录中的所有项
                if let Ok(entries) = fs::read_dir(&temp_dir) {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                let path = entry.path();
                                
                                // 尝试删除文件或目录
                                if path.is_file() {
                                    if let Ok(metadata) = entry.metadata() {
                                        let file_size = metadata.len();
                                        if fs::remove_file(&path).is_ok() {
                                            total_cleaned_size += file_size;
                                            deleted_files += 1;
                                        } else {
                                            skipped_files += 1;
                                            crate::logger::log_structured_warn(&log_ctx, &format!("跳过无法删除的文件: {}", path.display()));
                                        }
                                    }
                                } else if path.is_dir() {
                                    // 对于目录，尝试递归删除
                                    if let Err(err) = fs::remove_dir_all(&path) {
                                        skipped_files += 1;
                                        crate::logger::log_structured_warn(&log_ctx, &format!("跳过无法删除的目录 {}: {}", path.display(), err));
                                    } else {
                                        deleted_files += 1;
                                    }
                                }
                            },
                            Err(err) => {
                                skipped_files += 1;
                                crate::logger::log_structured_warn(&log_ctx, &format!("无法访问Temp目录项: {}", err));
                            }
                        }
                    }
                }
            } else {
                crate::logger::log_error("无法获取Temp目录");
            }
            
            // 发送完成状态消息
            if let Some(tx) = temp_tx {
                // 发送状态更新
                let status_message = format!(
                    "Temp目录清理完成: 已清理 {} 个文件，总计 {}，跳过 {} 个文件",
                    deleted_files,
                    crate::utils::format_size(total_cleaned_size),
                    skipped_files
                );
                let _ = tx.send((format!("__STATUS__{}", status_message), 0));

                // 发送完成标志
                let _ = tx.send(("__TEMP_CLEANUP_COMPLETE__".to_string(), 0));
            }
        });
    }

    // 新增：打开自定义文件夹选择对话框
    pub fn open_custom_folder_dialog(&mut self) {
        use native_dialog::FileDialog;
        
        logger::log_info("打开自定义文件夹选择对话框");
        
        // 打开文件夹选择对话框
        match FileDialog::new()
            .set_location("~")
            .show_open_single_dir()
        {
            Ok(Some(path)) => {
                logger::log_info(&format!("用户选择了自定义文件夹: {}", path.display()));
                self.set_custom_folder(path);
            }
            Ok(None) => {
                logger::log_info("用户取消了文件夹选择");
            }
            Err(e) => {
                logger::log_error(&format!("打开文件夹选择对话框失败: {}", e));
                self.status = Some(format!("打开文件夹选择对话框失败: {}", e));
            }
        }
    }

    // 新增：设置自定义文件夹并触发扫描
    pub fn set_custom_folder(&mut self, path: PathBuf) {
        use crate::utils::CUSTOM_FOLDER_PREFIX;
        
        self.custom_folder_path = Some(path.clone());
        self.selected_appdata_folder = format!("{}{}", CUSTOM_FOLDER_PREFIX, path.display());
        self.folder_data.clear();
        self.selected_folders.clear(); // 清空选中的文件夹集合
        self.is_scanning = true;
        self.status = Some("扫描自定义文件夹中...".to_string());
        
        if let Some(tx) = self.tx.clone() {
            crate::scanner::scan_custom_folder(tx, &path);
        }
    }
}
