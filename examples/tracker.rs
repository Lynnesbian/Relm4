use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use rand::prelude::IteratorRandom;
use relm4::{ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

const ICON_LIST: &[&str] = &[
    "bookmark-new-symbolic",
    "edit-copy-symbolic",
    "edit-cut-symbolic",
    "edit-find-symbolic",
    "starred-symbolic",
    "system-run-symbolic",
    "emoji-objects-symbolic",
    "emoji-nature-symbolic",
    "display-brightness-symbolic",
];

fn random_icon_name() -> &'static str {
    ICON_LIST
        .iter()
        .choose(&mut rand::rng())
        .expect("Could not choose a random icon")
}

// Returns a random icon different from the excluded one (avoids repeats).
fn gen_unique_icon(exclude: &'static str) -> &'static str {
    let mut rnd = random_icon_name();
    while rnd == exclude {
        rnd = random_icon_name()
    }
    rnd
}

#[derive(Debug)]
enum AppMsg {
    UpdateFirst,
    UpdateSecond,
}

// The track proc macro allows to easily track changes to different
// fields of the model
#[tracker::track]
struct AppModel {
    first_icon: &'static str,
    second_icon: &'static str,
    identical: bool,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            #[track(model.changed(AppModel::identical()))]
            set_class_active: ("identical", model.identical),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_all: 10,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Image {
                        set_pixel_size: 50,
                        #[track(model.changed(AppModel::first_icon()))]
                        set_icon_name: Some(model.first_icon),
                    },

                    gtk::Button {
                        set_label: "New random image",
                        connect_clicked => AppMsg::UpdateFirst,
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Image {
                        set_pixel_size: 50,
                        #[track(model.changed(AppModel::second_icon()))]
                        set_icon_name: Some(model.second_icon),
                    },

                    gtk::Button {
                        set_label: "New random image",
                        connect_clicked => AppMsg::UpdateSecond,
                    }
                },
            }
        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        // reset tracker value of the model
        self.reset();

        match msg {
            AppMsg::UpdateFirst => {
                self.set_first_icon(gen_unique_icon(self.first_icon));
            }
            AppMsg::UpdateSecond => {
                self.set_second_icon(gen_unique_icon(self.second_icon));
            }
        }
        self.set_identical(self.first_icon == self.second_icon);
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = AppModel {
            first_icon: random_icon_name(),
            second_icon: random_icon_name(),
            identical: false,
            tracker: 0,
        };

        model.identical = model.first_icon == model.second_icon;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.tracker");
    relm4::set_global_css(".identical { background: #00ad5c; }");

    app.run::<AppModel>(());
}
