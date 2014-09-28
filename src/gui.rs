use crate::{change_volume, Msg, VOLUME};
use gdk::{EventMask, ScrollDirection};
use gio::{prelude::*, ApplicationFlags};
use gtk::{prelude::*, Application, ApplicationWindow, Button, Inhibit, Label, Orientation};
use std::sync::atomic::Ordering;

/// Handle all the setting up of the gtk application.
pub fn run_gtk(jack_label: &str) {
    let application =
        Application::new(Some("com.github.jack-volume"), ApplicationFlags::NON_UNIQUE)
            .expect("failed to initialize GTK application");

    let window_title = format!("jack-volume: {}", jack_label);
    // Construct the user interface on activation.
    application.connect_activate(move |app| {
        let volume = VOLUME.load(Ordering::Relaxed);

        let window = ApplicationWindow::new(app);
        window.set_title(&window_title);
        window.set_default_size(100, 50);

        let container = gtk::Box::new(Orientation::Horizontal, 5);

        let down_btn = Button::new_with_label("down");
        down_btn.set_hexpand(true);
        let volume_txt = Label::new(Some(&show_volume(volume)));
        volume_txt.set_hexpand(true);
        let up_btn = Button::new_with_label("up");
        up_btn.set_hexpand(true);

        let volume_txt_for_down = volume_txt.clone();
        down_btn.connect_clicked(move |_| {
            let new_volume = change_volume(Msg::Down);
            volume_txt_for_down.set_text(&show_volume(new_volume));
        });

        let volume_txt_for_up = volume_txt.clone();
        up_btn.connect_clicked(move |_| {
            let new_volume = change_volume(Msg::Up);
            volume_txt_for_up.set_text(&show_volume(new_volume));
        });

        container.add(&down_btn);
        container.add(&volume_txt);
        container.add(&up_btn);

        window.add(&container);

        // Catch scroll events for quickly changing the volume with the mouse wheel.
        window.add_events(EventMask::SCROLL_MASK);
        let volume_txt_for_scroll = volume_txt.clone();
        window.connect_scroll_event(move |_, evt| {
            let msg = match evt.get_direction() {
                ScrollDirection::Up => Msg::Up,
                ScrollDirection::Down => Msg::Down,
                _ => return Inhibit(false),
            };
            let new_volume = change_volume(msg);
            volume_txt_for_scroll.set_text(&show_volume(new_volume));
            Inhibit(false)
        });
        // periodically check for updates to VOLUME. As far as I can tell this is the best way to
        // watch for changes. I would assume there is an event system for this as well, but I
        // haven't found it yet, and anyway it would be more complicated.
        let volume_txt_for_timeout = volume_txt.clone();
        // Time out is 50ms = 20Hz, which should be often enough to not be noticeable to the user.
        timeout_add(50, move || {
            let volume = VOLUME.load(Ordering::Relaxed);
            volume_txt_for_timeout.set_text(&show_volume(volume));
            Continue(true)
        });

        // display the window
        window.show_all();
    });

    application.run(&[]);
}

/// Convert the volume from a 0-127 range to a 0-100 range, and then return as a string
fn show_volume(vol: i8) -> String {
    (((vol as i64) * 100) / 127).to_string()
}
