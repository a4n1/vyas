use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::{MainThreadMarker, ns_string};

pub(super) fn install_menu() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);

    if unsafe { app.windowsMenu() }.is_some() {
        return;
    }

    let Some(main_menu) = (unsafe { app.mainMenu() }) else {
        return;
    };

    let window_menu = NSMenu::new(mtm);

    let item = NSMenuItem::new(mtm);
    unsafe { item.setTitle(ns_string!("Window")) };
    item.setSubmenu(Some(&window_menu));

    main_menu.addItem(&item);
    unsafe { app.setWindowsMenu(Some(&window_menu)) };
}
