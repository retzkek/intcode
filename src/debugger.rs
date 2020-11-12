use std::io;
use cursive::views::{TextView,TextArea,LinearLayout};
use super::Program;

pub fn debug(prog: &mut Program) -> io::Result<()> {
    let mut siv = cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    //siv.add_layer(TextView::new("Hello cursive! Press <q> to quit."));
    siv.add_fullscreen_layer(LinearLayout::vertical()
                  .child(TextArea::new().content("source here"))
                  .child(TextView::new("info here")));
    siv.run();
    Ok(())
}
