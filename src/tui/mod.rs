use cursive::Cursive;
//use cursive::align::HAlign;
//use cursive::traits::*;
//use cursive::views::{Dialog, DummyView, LinearLayout, TextView, SelectView};
//use cursive::theme::{Color, BaseColor, Effect, Style};
//use cursive::utils::markup::StyledString;
//use cursive::utils::span::SpannedString;
use cursive::menu::{MenuItem, MenuTree};
use cursive::event::{Event, Key};
use repositories::{Repositories, RepositoryExtensions};
use mru_list::MruList;
use paths;

pub fn run_cursive(repos: Repositories) {
    // If we managed to open at least 1, display it, else show the opening view.
    let mut siv = Cursive::default();
    create_menu_bar(&mut siv, &repos.mru);
   
    siv.run();

    // let mut select = SelectView::new().h_align(HAlign::Left);
    // for (i, repo) in repos.iter().enumerate() {
    //     select.add_item(repo.display_name(), i);
    // }
    // select.set_on_submit(on_mru_select);

    // siv.add_layer(Dialog::around(
    //     LinearLayout::vertical()
    //         .child(TextView::new(format!("{} {}", built_info::PKG_NAME, built_info::PKG_VERSION)).h_align(HAlign::Center))
    //         .child(DummyView.fixed_height(2))
    //         .child(TextView::new(in_bold("Choose a repository")))
    //         .child(DummyView.fixed_height(1))
    //         .child(select.scrollable().scroll_x(true).scroll_y(true).fixed_width(12)))
    // );
}

// fn make_menu_leaf(label: &str, callback: F)
//     where F: 'static + FnMut()
// {
// }

// fn on_mru_select(_siv: &mut Cursive, idx: &usize) {
//     siv.pop_layer();
//     let text = format!("{} is a great city!", idx);
//     siv.add_layer(
//         Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()),
// );
// }

// fn in_bold(s: &str) -> SpannedString<Style> {
//     let mut ss = StyledString::styled(s, Effect::Bold);
//     ss.append(StyledString::styled(" please", Style::from(Color::Light(BaseColor::Red))));
//     ss
// }

fn create_menu_bar(siv: &mut Cursive, mru: &MruList) {
    let _ = timer!("create_menu_bar");
    let file_menu = create_file_menu(siv, mru);
    siv.menubar().add_subtree("File", file_menu);
    let edit_menu = create_edit_menu(siv);
    siv.menubar().add_subtree("Edit", edit_menu);
    let view_menu = create_view_menu(siv);
    siv.menubar().add_subtree("View", view_menu);

    siv.set_autohide_menu(false);
    siv.add_global_callback(Key::F10, |s| s.select_menubar());
}

fn create_file_menu(siv: &mut Cursive, mru: &MruList) -> MenuTree {
    let mut menu = MenuTree::new();

    // TODO: So we have no way of doing C-S-something, unless something is something
    // like Key::F0 or Key::Right https://docs.rs/cursive/0.9.1/cursive/event/enum.Key.html
    // In addition, many other C-shortcuts do not seem to work!

    menu.add_leaf("New...      A-n", cb_file_new);
    siv.add_global_callback(Event::AltChar('n'), cb_file_new);
    menu.add_leaf("Open...     A-o", cb_file_open);
    siv.add_global_callback(Event::AltChar('o'), cb_file_open);
    menu.add_leaf("Clone...", cb_file_clone);

    if mru.len() > 0 {
        let mut recent_submenu = MenuTree::new();

        for (i, mru_item) in mru.iter().enumerate() {
            let mut label = if i < 9 {
                (i + 1).to_string()
            } else {
                " ".to_string()
            };

            label += &format!("  {}", paths::to_canon(mru_item).display());

            recent_submenu.add_leaf(label, |_| {});
        }

        menu.add_subtree("Recent", recent_submenu);
    }

    menu.add_delimiter();
    menu.add_leaf("Quit        C-q", cb_file_exit);                 // TODO: Doesn't work.
    siv.add_global_callback(Event::CtrlChar('q'), cb_file_exit);

    menu
}

fn create_edit_menu(siv: &mut Cursive) -> MenuTree {
    let mut menu = MenuTree::new();

    // Use chan-signal?

    menu.add_leaf("Undo           C-z", cb_edit_undo);
    siv.add_global_callback(Event::CtrlChar('z'), cb_edit_undo);        // TODO: Won't work. This is SIGSTOP. Use C-u?
    menu.add_leaf("Redo           C-y", cb_edit_redo);
    siv.add_global_callback(Event::CtrlChar('y'), cb_edit_redo);
    menu.add_delimiter();
    menu.add_leaf("Cut            C-x", cb_edit_cut);
    siv.add_global_callback(Event::CtrlChar('x'), cb_edit_cut);
    menu.add_leaf("Copy           C-c", cb_edit_copy);                  
    siv.add_global_callback(Event::CtrlChar('c'), cb_edit_copy);        // TODO: SIGINT. Won't work. Or SHIFT-HOME, or CS-c.
    menu.add_leaf("Paste          C-v", cb_edit_paste);                 
    siv.add_global_callback(Event::CtrlChar('v'), cb_edit_paste);       // Or SHIFT-INS, or CS-v.
    menu.add_leaf("Select All     C-a", cb_edit_select_all);
    siv.add_global_callback(Event::CtrlChar('a'), cb_edit_select_all);

    menu
}

fn create_view_menu(siv: &mut Cursive) -> MenuTree {
    let mut menu = MenuTree::new();

    menu.add_leaf("Refresh        F5", cb_view_refresh);
    siv.add_global_callback(Key::F5, cb_view_refresh);
    menu.add_delimiter();

    menu.add_leaf("Next Repo      C-n", cb_view_next_repo);
    siv.add_global_callback(Event::CtrlChar('n'), cb_view_next_repo);
    menu.add_leaf("Previous Repo  C-p", cb_view_previous_repo);
    siv.add_global_callback(Event::CtrlChar('p'), cb_view_previous_repo);

    menu.add_delimiter();
    menu.add_leaf("Main View      C-m", cb_view_main);
    siv.add_global_callback(Event::CtrlChar('m'), cb_view_main);
    menu.add_leaf("Log View       C-l", cb_view_log);
    siv.add_global_callback(Event::CtrlChar('l'), cb_view_log);

    menu
}

// Callbacks for each menu item.
fn cb_file_new(_siv: &mut Cursive) {
    info!("cb_file_new invoked...");
}

fn cb_file_open(_siv: &mut Cursive) {
    info!("cb_file_open invoked...");
}

fn cb_file_clone(_siv: &mut Cursive) {
    info!("cb_file_clone invoked...");
}

fn cb_file_exit(_siv: &mut Cursive) {
    info!("cb_file_exit invoked...");
    _siv.quit();
}

fn cb_edit_undo(_siv: &mut Cursive) {
    info!("cb_edit_undo invoked...");
}

fn cb_edit_redo(_siv: &mut Cursive) {
    info!("cb_edit_redo invoked...");
}

fn cb_edit_cut(_siv: &mut Cursive) {
    info!("cb_edit_cut invoked...");
}

fn cb_edit_copy(_siv: &mut Cursive) {
    info!("cb_edit_copy invoked...");
}

fn cb_edit_paste(_siv: &mut Cursive) {
    info!("cb_edit_paste invoked...");
}

fn cb_edit_select_all(_siv: &mut Cursive) {
    info!("cb_edit_select_all invoked...");
}

fn cb_view_refresh(_siv: &mut Cursive) {
    info!("cb_view_refresh invoked...");
}

fn cb_view_next_repo(_siv: &mut Cursive) {
    info!("cb_view_next_repo invoked...");
}

fn cb_view_previous_repo(_siv: &mut Cursive) {
    info!("cb_view_previous_repo invoked...");
}

fn cb_view_main(_siv: &mut Cursive) {
    info!("cb_view_main invoked...");
}

fn cb_view_log(_siv: &mut Cursive) {
    info!("cb_view_log invoked...");
}
