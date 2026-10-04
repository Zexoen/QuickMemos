use crate::api::{Memo, Visibility};
use crate::ui::markdown::render_markdown;
use crate::ui::{index_from_visibility, visibility_from_index};
use relm4::prelude::*;
use relm4::gtk::prelude::*;

#[derive(Debug)]
pub enum EditorOutput {
    Save { content: String, visibility: Visibility },
    Delete,
}

#[derive(Debug)]
pub enum EditorMsg {
    EditMemo(Memo),
    NewMemo,
}

pub struct MemoEditor {
    editing: bool,
}

#[relm4::component(pub)]
impl Component for MemoEditor {
    type Init = ();
    type Input = EditorMsg;
    type Output = EditorOutput;
    type CommandOutput = ();

    view! {
        #[root]
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 6,
            set_margin_top: 10,
            set_margin_bottom: 10,
            set_margin_start: 20,
            set_margin_end: 20,
            set_vexpand: true,
            set_hexpand: true,
            add_css_class: "editor-pane",

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                #[name(title_label)]
                gtk::Label {
                    set_label: "New memo",
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    add_css_class: "composer-title",
                },

                gtk::Image {
                    set_icon_name: Some("emblem-people-symbolic"),
                    set_pixel_size: 16,
                },

                #[name(vis_dropdown)]
                gtk::DropDown {
                    set_hexpand: false,
                },

                gtk::Button {
                    set_icon_name: "user-trash-symbolic",
                    set_tooltip_text: Some("Delete memo"),
                    set_css_classes: &["destructive-action", "flat"],
                    connect_clicked[sender] => move |_| {
                        let _ = sender.output(EditorOutput::Delete);
                    },
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 0,
                set_halign: gtk::Align::Start,
                add_css_class: "editor-tabs",

                #[name(write_toggle)]
                gtk::ToggleButton {
                    set_label: "Write",
                    set_active: true,
                    add_css_class: "tab-button",
                },

                #[name(preview_toggle)]
                gtk::ToggleButton {
                    set_label: "Preview",
                    add_css_class: "tab-button",
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                #[name(editor_scroller)]
                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_margin_top: 6,
                    set_min_content_width: 420,

                    #[name(editor_view)]
                    gtk::TextView {
                        set_wrap_mode: gtk::WrapMode::WordChar,
                        set_top_margin: 10,
                        set_bottom_margin: 10,
                        set_left_margin: 12,
                        set_right_margin: 12,
                        add_css_class: "editor-text",
                        set_hexpand: true,
                        set_vexpand: true,
                    },
                },

                #[name(preview_scroller)]
                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_margin_top: 6,
                    set_visible: false,
                    set_min_content_width: 420,

                    #[name(preview_label)]
                    gtk::Label {
                        set_markup: "",
                        set_wrap: true,
                        set_selectable: true,
                        set_xalign: 0.0,
                        set_yalign: 0.0,
                        set_margin_top: 10,
                        set_margin_bottom: 10,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        add_css_class: "preview-text",
                        connect_activate_link => |_, uri| {
                            let _ = std::process::Command::new("xdg-open").arg(uri).spawn();
                            gtk::glib::Propagation::Proceed
                        },
                    },
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::End,
                set_spacing: 6,

                gtk::Button {
                    set_label: "Save",
                    add_css_class: "suggested-action",
                    set_halign: gtk::Align::End,
                    connect_clicked[sender, editor_view, vis_dropdown] => move |_| {
                        let buffer = editor_view.buffer();
                        let (start, end) = buffer.bounds();
                        let content = buffer.text(&start, &end, true).to_string();
                        let visibility = visibility_from_index(vis_dropdown.selected() as usize);
                        let _ = sender.output(EditorOutput::Save { content, visibility });
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = MemoEditor { editing: false };
        let widgets = view_output!();

        {
            let visibility = gtk::StringList::new(
                &Visibility::all()
                    .iter()
                    .map(|v| v.label())
                    .collect::<Vec<&str>>(),
            );
            widgets.vis_dropdown.set_model(Some(&visibility));
            widgets.vis_dropdown.set_selected(0);
        }

        {
            let editor_scroller = widgets.editor_scroller.clone();
            let preview_scroller = widgets.preview_scroller.clone();
            widgets.write_toggle.connect_toggled(move |tb| {
                if tb.is_active() {
                    editor_scroller.set_visible(true);
                    preview_scroller.set_visible(false);
                }
            });
            let editor_scroller = widgets.editor_scroller.clone();
            let preview_scroller = widgets.preview_scroller.clone();
            let preview_label = widgets.preview_label.clone();
            let editor_buffer = widgets.editor_view.buffer().clone();
            widgets.preview_toggle.connect_toggled(move |tb| {
                if tb.is_active() {
                    editor_scroller.set_visible(false);
                    preview_scroller.set_visible(true);
                    refresh_preview(&preview_label, &editor_buffer);
                }
            });
        }

        {
            let preview_label = widgets.preview_label.clone();
            widgets.editor_view.buffer().connect_changed(move |buffer| {
                refresh_preview(&preview_label, buffer);
            });
        }

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
            EditorMsg::EditMemo(memo) => {
                self.editing = true;
                widgets.title_label.set_label("Edit memo");
                widgets.write_toggle.set_active(true);
                let buffer = widgets.editor_view.buffer();
                buffer.set_text(&memo.content);
                widgets
                    .vis_dropdown
                    .set_selected(index_from_visibility(&memo.visibility) as u32);
            }
            EditorMsg::NewMemo => {
                self.editing = false;
                widgets.title_label.set_label("New memo");
                widgets.write_toggle.set_active(true);
                let buffer = widgets.editor_view.buffer();
                buffer.set_text("");
                widgets.vis_dropdown.set_selected(0);
            }
        }
    }
}

fn refresh_preview(label: &gtk::Label, buffer: &gtk::TextBuffer) {
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, true).to_string();
    label.set_markup(&render_markdown(&text));
}