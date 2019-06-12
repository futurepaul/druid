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

//! Simple 7guis temperature converter example.
// https://eugenkiss.github.io/7guis/tasks/#temp

use druid_shell::platform::WindowBuilder;
use druid_shell::win_main;

use druid::widget::{Column, EventForwarder, Label, Padding, Row, TextBox };
use druid::{ UiMain, UiState};

use druid::Id;

//This feels like a bad first impression!
fn pad(widget: Id, state: &mut UiState) -> Id {
    Padding::uniform(5.0).ui(widget, state)
}

struct TempState {
    c: i32,
    f: i32
}

#[derive(Debug, Clone)]
enum TempAction {
  SetC(i32),
  SetF(i32)
}

impl TempState {
  fn action(&mut self, action: &TempAction) {
    match *action {
      TempAction::SetC(val) => {
          self.c = val;
          self.f = 32 + ((9.0 / 5.0 * val as f32).round() as i32);
      },
      TempAction::SetF(val) => {
          self.f = val;
          self.c = (5.0 / 9.0 * (val as f32 - 32.0)).round() as i32
      }
    }
  }
}

fn main() {
    druid_shell::init();

    let mut run_loop = win_main::RunLoop::new();
    let mut builder = WindowBuilder::new();
    let mut state = UiState::new();

    let mut temp = TempState { c: 0, f: 0 };

    let c_box = TextBox::new("0", 50.).ui(&mut state);
    let c_box_padded = pad(c_box, &mut state);

    let c_label = Label::new("Celsius = ").ui(&mut state);
    let c_label_padded = pad(c_label, &mut state);

    let f_box = TextBox::new("0", 50.).ui(&mut state);
    let f_box_padded = pad(f_box, &mut state);

    let f_label = Label::new("Fahrenheit").ui(&mut state);
    let f_label_padded = pad(f_label, &mut state);

    let row = Row::new();

    let column = Column::new();
    let row = row.ui(&[c_box_padded, c_label_padded, f_box_padded, f_label_padded], &mut state);
    let row = pad(row, &mut state);

    let panel = column.ui(&[row], &mut state);

    state.add_listener(f_box, move |value: &mut String, mut ctx| {
      match value.trim().parse() {
        Ok(num) => ctx.poke_up(&mut TempAction::SetF(num)),
        Err(_) => false 
      };
    });

    state.add_listener(c_box, move |value: &mut String, mut ctx| {
      match value.trim().parse() {
        Ok(num) => ctx.poke_up(&mut TempAction::SetC(num)),
        Err(_) => false 
      };
    });

    let forwarder = EventForwarder::<TempAction>::new().ui(row, &mut state);

    state.add_listener(forwarder, move |action: &mut TempAction, mut ctx| {

      // Calculate the new state
      temp.action(action);

      // Cause a redraw with the new state
      ctx.poke(f_box, &mut temp.f.to_string());
      ctx.poke(c_box, &mut temp.c.to_string());

    });

    state.set_root(panel);
    builder.set_handler(Box::new(UiMain::new(state)));
    builder.set_title("Temperature Converter");
    let window = builder.build().expect("built window");
    window.show();
    run_loop.run();
}
