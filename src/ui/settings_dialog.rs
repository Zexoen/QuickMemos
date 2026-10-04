use crate::config::AppConfig;
use relm4::prelude::*;
use relm4::gtk::prelude::*;

#[derive(Debug)]
pub enum SettingsOutput {
    Saved { server_url: String, access_token: String },
    Cancelled,
}

#[derive(Debug)]
pub enum SettingsInput {
    Show,
    Hide,
}

pub struct SettingsDialog {
    root: gtk::Window,
    server_url: String,
    access_token: String,
}

#[relm4::component(pub)]
impl Component for SettingsDialog {
    type Init = (gtk::ApplicationWindow, AppConfig);
    type Input = SettingsInput;
    type Output = SettingsOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Window {
            set_title: Some("QuickMemos Settings"),
            set_default_size: (420, 200),
            set_modal: true,
            set_hide_on_close: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,
                set_margin_all: 16,

                gtk::Label {
                    set_label: "Memos Server URL",
                    set_halign: gtk::Align::Start,
                },

                #[name(server_entry)]
                gtk::Entry {
                    set_placeholder_text: Some("https://memos.example.com"),
                },

                gtk::Label {
                    set_label: "Access Token",
                    set_halign: gtk::Align::Start,
                },

                #[name(token_entry)]
                gtk::Entry {
                    set_visibility: false,
                    set_placeholder_text: Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."),
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::End,
                    set_spacing: 6,
                    set_margin_top: 12,

                    gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(SettingsOutput::Cancelled);
                        },
                    },

                    gtk::Button {
                        set_label: "Save",
                        add_css_class: "suggested-action",
                        connect_clicked[sender, server_entry, token_entry] => move |_| {
                            let server_url = server_entry.text().to_string();
                            let access_token = token_entry.text().to_string();
                            let _ = sender.output(SettingsOutput::Saved { server_url, access_token });
                        },
                    },
                },
            },
        }
    }

    fn init(
        (transient_for, config): Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        root.set_transient_for(Some(&transient_for));

        let model = SettingsDialog {
            root: root.clone(),
            server_url: config.server_url.clone(),
            access_token: config.access_token.clone(),
        };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: relm4::ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SettingsInput::Show => {
                self.root.show();
                widgets.server_entry.set_text(&self.server_url);
                widgets.token_entry.set_text(&self.access_token);
            }
            SettingsInput::Hide => {
                self.root.hide();
            }
        }
    }
}
