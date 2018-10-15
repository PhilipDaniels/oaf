use cursive::Cursive;
//use cursive::align::HAlign;
//use cursive::traits::*;
//use cursive::views::{Dialog, DummyView, LinearLayout, TextView, SelectView};
//use cursive::theme::{Color, BaseColor, Effect, Style};
//use cursive::utils::markup::StyledString;
//use cursive::utils::span::SpannedString;
use cursive::menu::{MenuItem, MenuTree};
use cursive::event::{Event, Key};
use repositories::Repositories;
use mru_list::OafMruList;

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

fn create_menu_bar(siv: &mut Cursive, mru: &OafMruList) {
    let file_menu = create_file_menu(siv, mru);
    siv.menubar().add_subtree("File", file_menu);

    siv.set_autohide_menu(false);
    siv.add_global_callback(Key::F10, |s| s.select_menubar());
}

fn create_file_menu(siv: &mut Cursive, mru: &OafMruList) -> MenuTree {
    let mut menu = MenuTree::new();

    menu.add_leaf("New...      C-n", cb_file_new);
    siv.add_global_callback(Event::CtrlChar('n'), cb_file_new);
    menu.add_leaf("Open...     C-o", cb_file_open);
    siv.add_global_callback(Event::CtrlChar('o'), cb_file_open);
    menu.add_leaf("Clone...", cb_file_clone);

    if mru.len() > 0 {
        let recent_submenu = MenuTree::new();

        for mru_repp in mru.iter() {

        }

        menu.add_subtree("Recent", recent_submenu);
    }

    menu.add_delimiter();
    menu.add_leaf("Exit", cb_file_exit);

    menu
}

// fn make_menu_leaf(label: &str, callback: F)
//     where F: 'static + FnMut()
// {

// }

// Callbacks for each menu item.
fn cb_file_new(siv: &mut Cursive) {
    info!("cb_file_new invoked...");
}

fn cb_file_open(siv: &mut Cursive) {
    info!("cb_file_open invoked...");
}

fn cb_file_clone(siv: &mut Cursive) {
    info!("cb_file_clone invoked...");
}

fn cb_file_exit(siv: &mut Cursive) {
    info!("cb_file_exit invoked...");
    siv.quit();
}





// fn on_mru_select(siv: &mut Cursive, idx: &usize) {
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
