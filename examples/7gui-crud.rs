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

use druid::widget::{ScrollColumn, Column, EventForwarder, Label, Padding, Row, TextBox };
use druid::{ UiMain, UiState};

use druid::Id;

fn main() {
    druid_shell::init();

    let mut run_loop = win_main::RunLoop::new();
    let mut builder = WindowBuilder::new();
    let mut state = UiState::new();

    let label1 = Label::new("thing1").ui(&mut state);
    let label2 = Label::new("thing2").ui(&mut state);
    let column = Column::new().ui(&[label1, label2], &mut state);
    let scroll_column = ScrollColumn::new().ui(&[column], &mut state);

    state.set_root(scroll_column);

    builder.set_handler(Box::new(UiMain::new(state)));
    builder.set_title("Temperature Converter");
    let window = builder.build().expect("built window");
    window.show();
    run_loop.run();
}
