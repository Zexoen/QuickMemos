use crate::api::{ApiError, MemosClient};
use crate::config::AppConfig;
use crate::ui::{
    header::{Header, HeaderOutput},
    memo_editor::{EditorMsg, EditorOutput, MemoEditor},
    memo_row::{MemoRow, RowOutput},
    settings_dialog::{SettingsDialog, SettingsInput, SettingsOutput},
    MAX_PAGE_SIZE,
};
use relm4::prelude::*;
use relm4::gtk::prelude::*;

pub struct App {
    config: AppConfig,
    client: Option<MemosClient>,
    memos: Vec<crate::api::Memo>,
    memo_list: FactoryVecDeque<MemoRow>,
    editing_memo: Option<String>,
    status: String,
    loading: bool,
    show_archived: bool,
    next_page_token: String,
    query: String,
    header: Controller<Header>,
    editor: Controller<MemoEditor>,
    settings: Controller<SettingsDialog>,
}

#[derive(Debug)]
pub enum AppMsg {
    NewMemo,
    Refresh,
    SearchChanged(String),
    SetShowArchived(bool),
    SettingsClicked,
    SettingsDone(SettingsOutput),
    EditorDone(EditorOutput),
    RowAction(RowOutput),
    Fetched(Result<crate::api::ListMemosResponse, ApiError>),
}

#[relm4::component(async, pub)]
impl AsyncComponent for App {
    type Init = AppConfig;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_title: Some("QuickMemos"),
            set_default_size: (1100, 680),
            set_titlebar: Some(model.header.widget()),

            #[wrap(Some)]
            set_child = &gtk::Paned {
                set_wide_handle: true,
                set_position: 200,
                set_vexpand: true,

                #[wrap(Some)]
                set_start_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_top: 18,
                    set_margin_bottom: 12,
                    set_margin_start: 12,
                    set_margin_end: 12,
                    set_halign: gtk::Align::Fill,
                    add_css_class: "sidebar",

                    gtk::Label {
                        set_label: "QuickMemos",
                        add_css_class: "sidebar-brand",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Separator {
                        set_margin_top: 4,
                        set_margin_bottom: 4,
                    },

                    gtk::Button {
                        set_label: "Memos",
                        set_halign: gtk::Align::Fill,
                        set_css_classes: &["flat", "nav-button"],
                        connect_clicked[sender] => move |_| {
                            let _ = sender.input(AppMsg::SetShowArchived(false));
                        },
                    },

                    gtk::Button {
                        set_label: "Archived",
                        set_halign: gtk::Align::Fill,
                        set_css_classes: &["flat", "nav-button"],
                        connect_clicked[sender] => move |_| {
                            let _ = sender.input(AppMsg::SetShowArchived(true));
                        },
                    },

                    gtk::Separator {
                        set_margin_top: 4,
                        set_margin_bottom: 4,
                    },

                    gtk::Button {
                        set_label: "Settings",
                        set_halign: gtk::Align::Fill,
                        set_css_classes: &["flat", "nav-button"],
                        connect_clicked[sender] => move |_| {
                            let _ = sender.input(AppMsg::SettingsClicked);
                        },
                    },

                    gtk::Box { set_vexpand: true },

                    gtk::Label {
                        set_label: "I'm using QuickMemos",
                        add_css_class: "sidebar-footer",
                        set_halign: gtk::Align::Start,
                    },
                },

                #[wrap(Some)]
                set_end_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 0,
                    set_hexpand: true,
                    set_vexpand: true,

                    gtk::Label {
                        #[watch]
                        set_label: &model.status,
                        set_halign: gtk::Align::Start,
                        set_margin_top: 6,
                        set_margin_bottom: 4,
                        set_margin_start: 12,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        add_css_class: "status-label",
                    },

                    gtk::Paned {
                        set_wide_handle: true,
                        set_position: 480,
                        set_vexpand: true,

                        #[wrap(Some)]
                        set_start_child = &gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            #[local_ref]
                            memo_list -> gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 2,
                                set_vexpand: true,
                            },
                        },

                        #[wrap(Some)]
                        set_end_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: true,
                            add_css_class: "editor-pane",

                            #[local_ref]
                            editor_root -> gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_hexpand: true,
                                set_vexpand: true,
                            },
                        },
                    },
                },
            },

            connect_close_request[sender] => move |_| {
                let _ = &sender;
                relm4::main_application().quit();
                gtk::glib::Propagation::Proceed
            },
        }
    }

    async fn init(
        config: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        {
            let provider = gtk::CssProvider::new();
            provider.load_from_data(include_str!("styles.css"));
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().expect("No display available"),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let client = if !config.server_url.is_empty() && !config.access_token.is_empty() {
            Some(MemosClient::new(&config.server_url, &config.access_token))
        } else {
            None
        };

        let header: Controller<Header> = Header::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                HeaderOutput::NewMemo => AppMsg::NewMemo,
                HeaderOutput::Refresh => AppMsg::Refresh,
                HeaderOutput::SearchChanged(text) => AppMsg::SearchChanged(text),
            });

        let memo_list: FactoryVecDeque<MemoRow> = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), AppMsg::RowAction);

        let editor: Controller<MemoEditor> = MemoEditor::builder()
            .launch(())
            .forward(sender.input_sender(), AppMsg::EditorDone);

        let settings: Controller<SettingsDialog> = SettingsDialog::builder()
            .launch((root.clone(), config.clone()))
            .forward(sender.input_sender(), AppMsg::SettingsDone);

        let mut model = App {
            config,
            client,
            memos: Vec::new(),
            memo_list,
            editing_memo: None,
            status: String::new(),
            loading: false,
            show_archived: false,
            next_page_token: String::new(),
            query: String::new(),
            header,
            editor,
            settings,
        };

        model.fetch_memos(sender.input_sender()).await;

        if model.client.is_none() {
            model.status = "Not configured. Please open Settings.".into();
            model.settings.emit(SettingsInput::Show);
        }

        let memo_list = model.memo_list.widget();
        let editor_root = model.editor.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::NewMemo => {
                self.editing_memo = None;
                self.editor.emit(EditorMsg::NewMemo);
            }
            AppMsg::Refresh => {
                self.fetch_memos(sender.input_sender()).await;
            }
            AppMsg::SearchChanged(query) => {
                self.query = query;
                self.fetch_memos(sender.input_sender()).await;
            }
            AppMsg::SetShowArchived(show) => {
                self.show_archived = show;
                self.fetch_memos(sender.input_sender()).await;
            }
            AppMsg::SettingsClicked => {
                self.settings.emit(SettingsInput::Show);
            }
            AppMsg::SettingsDone(SettingsOutput::Saved {
                server_url,
                access_token,
            }) => {
                self.config.server_url = server_url;
                self.config.access_token = access_token;
                if let Err(e) = self.config.save() {
                    self.set_status(format!("Failed to save settings: {e}"));
                    return;
                }
                self.rebuild_client();
                self.fetch_memos(sender.input_sender()).await;
                self.settings.emit(SettingsInput::Hide);
            }
            AppMsg::SettingsDone(SettingsOutput::Cancelled) => {
                self.settings.emit(SettingsInput::Hide);
            }
            AppMsg::RowAction(RowOutput::Selected(index)) => {
                let i = index.current_index();
                if let Some(memo) = self.memos.get(i) {
                    self.editing_memo = Some(crate::ui::memo_id(&memo.name));
                    self.editor.emit(EditorMsg::EditMemo(memo.clone()));
                }
            }
            AppMsg::RowAction(RowOutput::TogglePin(index)) => {
                let i = index.current_index();
                if let Some(memo) = self.memos.get(i).cloned() {
                    let memo_id = crate::ui::memo_id(&memo.name);
                    let new_pinned = !memo.pinned;
                    if let Some(client) = &self.client {
                        match client
                            .update_memo(&memo_id, None, None, Some(new_pinned))
                            .await
                        {
                            Ok(_) => {
                                if let Some(m) = self.memos.get_mut(i) {
                                    m.pinned = new_pinned;
                                }
                                self.rebuild_list();
                                self.set_status(format!(
                                    "Memo {}",
                                    if new_pinned { "pinned" } else { "unpinned" }
                                ));
                            }
                            Err(e) => self.set_status(format!("Failed to toggle pin: {e}")),
                        }
                    }
                }
            }
            AppMsg::RowAction(RowOutput::Delete(index)) => {
                let i = index.current_index();
                if let Some(memo) = self.memos.get(i).cloned() {
                    let memo_id = crate::ui::memo_id(&memo.name);
                    if let Some(client) = &self.client {
                        match client.delete_memo(&memo_id).await {
                            Ok(()) => {
                                self.memos.remove(i);
                                if self.editing_memo.as_deref() == Some(&memo_id) {
                                    self.editing_memo = None;
                                    self.editor.emit(EditorMsg::NewMemo);
                                }
                                self.rebuild_list();
                                self.set_status("Memo deleted");
                            }
                            Err(e) => self.set_status(format!("Failed to delete: {e}")),
                        }
                    }
                }
            }
            AppMsg::EditorDone(EditorOutput::Save {
                content,
                visibility,
            }) => {
                if content.trim().is_empty() {
                    self.set_status("Cannot save empty memo");
                    return;
                }
                if let Some(client) = &self.client {
                    if let Some(editing_id) = self.editing_memo.clone() {
                        match client
                            .update_memo(&editing_id, Some(&content), Some(&visibility), None)
                            .await
                        {
                            Ok(updated) => {
                                if let Some(m) = self
                                    .memos
                                    .iter_mut()
                                    .find(|m| crate::ui::memo_id(&m.name) == editing_id)
                                {
                                    m.content = updated.content;
                                    m.visibility = updated.visibility;
                                    m.update_time = updated.update_time;
                                }
                                self.rebuild_list();
                                self.set_status("Memo updated");
                            }
                            Err(e) => self.set_status(format!("Failed to update: {e}")),
                        }
                    } else {
                        match client.create_memo(&content, Some(&visibility)).await {
                            Ok(_) => {
                                self.fetch_memos(sender.input_sender()).await;
                                self.set_status("Memo created");
                                self.editor.emit(EditorMsg::NewMemo);
                            }
                            Err(e) => self.set_status(format!("Failed to create: {e}")),
                        }
                    }
                } else {
                    self.set_status("Not configured. Open Settings first.");
                }
            }
            AppMsg::EditorDone(EditorOutput::Delete) => {
                if let Some(editing_id) = self.editing_memo.clone() {
                    if let Some(client) = &self.client {
                        match client.delete_memo(&editing_id).await {
                            Ok(()) => {
                                let idx = self
                                    .memos
                                    .iter()
                                    .position(|m| crate::ui::memo_id(&m.name) == editing_id);
                                if let Some(i) = idx {
                                    self.memos.remove(i);
                                }
                                self.editing_memo = None;
                                self.editor.emit(EditorMsg::NewMemo);
                                self.rebuild_list();
                                self.set_status("Memo deleted");
                            }
                            Err(e) => self.set_status(format!("Failed to delete: {e}")),
                        }
                    }
                }
            }
            AppMsg::Fetched(result) => match result {
                Ok(resp) => {
                    self.memos = resp.memos;
                    self.next_page_token = resp.next_page_token;
                    self.rebuild_list();
                    self.set_status(format!("{} memo(s)", self.memos.len()));
                }
                Err(e) => self.set_status(format!("Failed to load: {e}")),
            },
        }
    }
}

impl App {
    fn rebuild_client(&mut self) {
        self.client = if !self.config.server_url.is_empty() && !self.config.access_token.is_empty()
        {
            Some(MemosClient::new(&self.config.server_url, &self.config.access_token))
        } else {
            None
        };
    }

    fn rebuild_list(&mut self) {
        let mut guard = self.memo_list.guard();
        guard.clear();
        for memo in &self.memos {
            guard.push_back(memo.clone());
        }
    }

    fn set_status(&mut self, text: impl Into<String>) {
        self.status = text.into();
    }

    async fn fetch_memos(&mut self, sender: &relm4::Sender<AppMsg>) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.status = "Loading...".into();

        let filter = if self.query.is_empty() {
            None
        } else {
            let escaped = self.query.replace('"', "\\\"");
            Some(format!("content.contains(\"{escaped}\")"))
        };
        let state = if self.show_archived {
            "ARCHIVED"
        } else {
            "NORMAL"
        };

        let result = match &self.client {
            Some(client) => {
                client
                    .list_memos(MAX_PAGE_SIZE, "", filter.as_deref(), state)
                    .await
            }
            None => Err(ApiError::Other(
                "Not configured. Open Settings.".into(),
            )),
        };

        self.loading = false;
        let _ = sender.send(AppMsg::Fetched(result));
    }
}