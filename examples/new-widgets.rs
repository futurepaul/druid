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

//! Simple textbox example.

use druid_shell::platform::WindowBuilder;
use druid_shell::win_main;

use druid::widget::{Checkbox, Column, Label, Padding, ProgressBar, Row, Slider, TextBox};
use druid::{UiMain, UiState};

use druid::Id;

fn pad(widget: Id, state: &mut UiState) -> Id {
    Padding::uniform(5.0).ui(widget, state)
}

fn main() {
    druid_shell::init();
    let mut run_loop = win_main::RunLoop::new();
    let mut builder = WindowBuilder::new();

    let mut state = UiState::new();

    // We declare text_box and text_box_padded separately so we can sent events to text_box
    // If we poked text_box_padded it would fail silently
    let text_box = TextBox::new(Some("1.00".to_string()), 50.).ui(&mut state);
    let text_box_padded = pad(text_box, &mut state);

    let slider = Slider::new(1.0).ui(&mut state);
    let slider_padded = pad(slider, &mut state);

    let label = Label::new("1.00").ui(&mut state);
    let label_padded = pad(label, &mut state);

    let progress_bar = ProgressBar::new(1.0).ui(&mut state);
    let progress_bar_padded = pad(progress_bar, &mut state);

    let checkbox_label = Label::new("true").ui(&mut state);
    let checkbox_label_padded = pad(checkbox_label, &mut state);

    let checkbox = Checkbox::new(true).ui(&mut state);
    let checkbox_padded = pad(checkbox, &mut state);

    let mut row_1 = Row::new();
    let mut row_2 = Row::new();
    let mut row_3 = Row::new();

    row_1.set_flex(slider_padded, 1.0);
    row_2.set_flex(progress_bar_padded, 1.0);
    row_3.set_flex(checkbox_label_padded, 1.0);

    let row_1 = row_1.ui(&[slider_padded, label_padded], &mut state);
    let row_2 = row_2.ui(&[progress_bar_padded], &mut state);
    let row_3 = row_3.ui(&[checkbox_padded, checkbox_label_padded], &mut state);

    let column = Column::new();

    let panel = column.ui(&[text_box_padded, row_1, row_2, row_3], &mut state);

    state.add_listener(slider, move |value: &mut f64, mut ctx| {
        ctx.poke(progress_bar, value);
        ctx.poke(text_box, &mut format!("{:.2}", value));
        ctx.poke(label, &mut format!("{:.2}", value));

        if *value == 1.0 as f64 {
            ctx.poke(checkbox, &mut true);
            ctx.poke(checkbox_label, &mut "true".to_string());
        } else {
            ctx.poke(checkbox, &mut false);
            ctx.poke(checkbox_label, &mut "false".to_string());
        }
    });

    state.add_listener(checkbox, move |value: &mut bool, mut ctx| {
        ctx.poke(checkbox_label, &mut value.to_string());
    });

    state.set_root(panel);
    builder.set_handler(Box::new(UiMain::new(state)));
    builder.set_title("New widgets");
    let window = builder.build().expect("built window");
    window.show();
    run_loop.run();
}
