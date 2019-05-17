// Copyright 2019 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Simple 7guis counter example.
// https://eugenkiss.github.io/7guis/tasks#counter

use druid_shell::platform::WindowBuilder;
use druid_shell::win_main;

use druid::widget::{Button, Column, EventForwarder, KeyListener, Label, Padding, Row, Slider, TextBox, ProgressBar};
use druid::{KeyEvent, KeyVariant, UiMain, UiState};

use druid::Id;

//This feels like a bad first impression!
fn pad(widget: Id, state: &mut UiState) -> Id {
    Padding::uniform(5.0).ui(widget, state)
}

struct CounterState {
  count: i32
}

#[derive(Debug, Clone)]
enum CounterAction {
  Increment,
  Set(i32)
}

impl CounterState {
  fn action(&mut self, action: &CounterAction) {
    match *action {
      CounterAction::Increment => self.count += 1,
      CounterAction::Set(num) => self.count = num
    }
  }
}

fn main() {
    druid_shell::init();

    let mut run_loop = win_main::RunLoop::new();
    let mut builder = WindowBuilder::new();
    let mut state = UiState::new();


    let mut counter = CounterState { count: 0 };

    let text_box = TextBox::new("1", 50.).ui(&mut state);
    let text_box_padded = pad(text_box, &mut state);

    let button = Button::new("Count".to_string()).ui(&mut state);
    let button_padded = pad(button, &mut state);



    let mut row = Row::new();
    row.set_flex(text_box_padded, 1.0);

    let mut column = Column::new();
    let row = row.ui(&[text_box_padded, button_padded], &mut state);
    let row = pad(row, &mut state);
    column.set_flex(row, 1.0);

    let panel = column.ui(&[row], &mut state);

    state.add_listener(button, move |_: &mut bool, mut ctx| {
      counter.action(&CounterAction::Increment);
      ctx.poke(text_box, &mut counter.count.to_string());
    });

    //LIFETIME PROBS
    // state.add_listener(text_box, move |value: &mut String, mut ctx| {
    //   let parsed_value: i32 = match value.trim().parse() {
    //     Ok(num) => num,
    //     Err(_) => 0
    //   };
    //   counter.action(&CounterAction::Set(parsed_value));
    // });

    state.set_root(panel);
    builder.set_handler(Box::new(UiMain::new(state)));
    builder.set_title("Counter");
    let window = builder.build().expect("built window");
    window.show();
    run_loop.run();
}
