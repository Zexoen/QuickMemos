use crate::api::Memo;
use crate::ui::markdown::render_markdown;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::prelude::*;
use relm4::gtk::prelude::*;

#[derive(Debug)]
pub enum RowOutput {
    Selected(DynamicIndex),
    TogglePin(DynamicIndex),
    Delete(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for MemoRow {
    type Init = Memo;
    type Input = ();
    type Output = RowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 6,
            set_margin_top: 4,
            set_margin_bottom: 4,
            add_css_class: "memo-card",

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_bottom: 2,

                gtk::Label {
                    set_label: &avatar_text(&self.memo.creator),
                    add_css_class: "avatar",
                    set_width_request: 34,
                    set_height_request: 34,
                    set_xalign: 0.5,
                    set_yalign: 0.5,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 0,

                    gtk::Label {
                        set_label: &user_label(&self.memo.creator),
                        set_halign: gtk::Align::Start,
                        add_css_class: "memo-username",
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &crate::ui::format_time(&self.memo.create_time),
                        set_halign: gtk::Align::Start,
                        add_css_class: "memo-date",
                    },
                },

                gtk::Box {
                    set_hexpand: true,
                },

                gtk::Image {
                    #[watch]
                    set_icon_name: Some(self.memo.visibility.icon_name()),
                    set_pixel_size: 13,
                },

                gtk::Image {
                    #[watch]
                    set_icon_name: Some(if self.memo.pinned {
                        "starred-symbolic"
                    } else {
                        "non-starred-symbolic"
                    }),
                    set_pixel_size: 13,
                },
            },

            gtk::Button {
                set_halign: gtk::Align::Fill,
                set_hexpand: true,
                set_has_frame: false,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,

                    #[name(content_label)]
                    gtk::Label {
                        #[watch]
                        set_markup: &render_markdown(&self.memo.content),
                        set_wrap: true,
                        set_lines: 5,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_xalign: 0.0,
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        set_selectable: true,
                        add_css_class: "memo-content",
                        connect_activate_link[sender, index] => move |_, uri| {
                            let _ = sender.output(RowOutput::Selected(index.clone()));
                            activate_link(uri);
                            gtk::glib::Propagation::Proceed
                        },
                    },
                },

                connect_clicked[sender, index] => move |_| {
                    let _ = sender.output(RowOutput::Selected(index.clone()));
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_halign: gtk::Align::End,
                add_css_class: "card-actions",

                gtk::Button {
                    #[watch]
                    set_icon_name: if self.memo.pinned {
                        "starred-symbolic"
                    } else {
                        "non-starred-symbolic"
                    },
                    set_tooltip_text: Some(if self.memo.pinned { "Unpin" } else { "Pin" }),
                    set_css_classes: &["flat"],
                    connect_clicked[sender, index] => move |_| {
                        let _ = sender.output(RowOutput::TogglePin(index.clone()));
                    },
                },

                gtk::Button {
                    set_icon_name: "user-trash-symbolic",
                    set_tooltip_text: Some("Delete"),
                    set_css_classes: &["destructive-action", "flat"],
                    connect_clicked[sender, index] => move |_| {
                        let _ = sender.output(RowOutput::Delete(index.clone()));
                    },
                },
            },
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { memo: value }
    }
}

pub struct MemoRow {
    memo: Memo,
}

fn avatar_text(creator: &str) -> String {
    let digits: String = creator.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "M".into())
}

fn user_label(creator: &str) -> String {
    let digits: String = creator.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        "Memos".to_string()
    } else {
        format!("User {digits}")
    }
}

fn activate_link(uri: &str) {
    let _ = std::process::Command::new("xdg-open").arg(uri).spawn();
}