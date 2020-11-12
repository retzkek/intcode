use super::{Input, Instruction, Int, Output, Program};
use cursive::theme::{Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::view::{scroll::ScrollStrategy, Nameable, SizeConstraint};
use cursive::views::{Dialog, LinearLayout, Panel, ResizedView, ScrollView, TextContent, TextView};
use cursive::Cursive;
use std::io;

struct Debugger {
    program: Program,
    cur_addr: usize,
}

impl Debugger {
    fn cur_len(&self) -> usize {
        self.program.peek(self.cur_addr as Int).op().len()
    }

    pub fn code_string(&self) -> StyledString {
        let mut code = StyledString::new();
        for (i, int) in self.program.mem.iter().enumerate() {
            code.append(StyledString::styled(
                format!["{:<4} ", int],
                if i >= self.cur_addr && i < (self.cur_addr + self.cur_len()) {
                    Style::from(Effect::Reverse)
                } else {
                    Style::none()
                },
            ));
        }
        code
    }

    pub fn stack_string(&self) -> StyledString {
        let mut code = StyledString::new();
        for entry in self.program.stack.iter() {
            code.append(StyledString::plain(format![
                "{:<4}: {:<4}: {}\n",
                entry.address,
                entry.value,
                entry.value.op()
            ]));
        }
        code
    }

    pub fn step(&mut self) {
        match self.program.step(
            self.cur_addr as Int,
            false,
            &mut Input::None,
            &mut Output::None,
        ) {
            Ok(-1) => {}
            Ok(r) => self.cur_addr = r as usize,
            Err(e) => eprintln!("{}", e),
        };
    }
}

pub fn debug(prog: Program) -> io::Result<()> {
    let d = Debugger {
        program: prog,
        cur_addr: 0,
    };

    let mut siv = cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('n', |s| {
        let d = s.user_data::<Debugger>().unwrap();
        d.step();
        let code = d.code_string();
        let stack = d.stack_string();
        s.call_on_name("code", |v: &mut TextView| {
            v.set_content(code);
        });
        s.call_on_name("stack", |v: &mut TextView| {
            v.set_content(stack);
        });
    });

    siv.add_fullscreen_layer(
        LinearLayout::vertical()
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Fixed(30),
                Panel::new(TextView::new(d.code_string()).with_name("code")).title("Code"),
            ))
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                LinearLayout::horizontal()
                    .child(ResizedView::new(
                        SizeConstraint::Full,
                        SizeConstraint::Full,
                        Panel::new(
                            ScrollView::new(TextView::new("[empty]").with_name("stack"))
                                .scroll_strategy(ScrollStrategy::StickToBottom),
                        )
                        .title("Stack"),
                    ))
                    .child(ResizedView::new(
                        SizeConstraint::Full,
                        SizeConstraint::Full,
                        Panel::new(TextView::new("[TODO]")).title("Instruction"),
                    )),
            ))
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Fixed(1),
                TextView::new("q:quit n:next"),
            )),
    );

    siv.set_user_data(d);
    siv.run();
    Ok(())
}
