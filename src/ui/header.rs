use relm4::prelude::*;
use relm4::gtk::prelude::*;

#[derive(Debug)]
pub enum HeaderOutput {
    NewMemo,
    Refresh,
    SearchChanged(String),
}

pub struct Header;

#[relm4::component(pub)]
impl SimpleComponent for Header {
    type Init = ();
    type Input = ();
    type Output = HeaderOutput;

    view! {
        #[root]
        gtk::HeaderBar {
            set_show_title_buttons: true,

            pack_start = &gtk::Button {
                set_icon_name: "list-add-symbolic",
                set_tooltip_text: Some("New memo"),
                connect_clicked[sender] => move |_| {
                    let _ = sender.output(HeaderOutput::NewMemo);
                },
            },

            pack_start = &gtk::Button {
                set_icon_name: "view-refresh-symbolic",
                set_tooltip_text: Some("Refresh"),
                connect_clicked[sender] => move |_| {
                    let _ = sender.output(HeaderOutput::Refresh);
                },
            },

            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,
                set_halign: gtk::Align::Center,

                gtk::Label {
                    set_label: "QuickMemos",
                    add_css_class: "title",
                },

                gtk::Label {
                    set_label: "Memos Client",
                    add_css_class: "subtitle",
                },
            },

            pack_start = &gtk::SearchEntry {
                set_placeholder_text: Some("Search memos..."),
                connect_search_changed[sender] => move |entry| {
                    let _ = sender.output(HeaderOutput::SearchChanged(entry.text().to_string()));
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Header;
        let widgets = view_output!();
        relm4::ComponentParts { model, widgets }
    }
}