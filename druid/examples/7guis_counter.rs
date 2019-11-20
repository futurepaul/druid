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

use druid::widget::{Align, Button, DynLabel, Row};
use druid::{AppLauncher, Widget, WindowDesc};

fn main() {
    let main_window = WindowDesc::new(ui_builder);
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<u32> {
    let label = DynLabel::new(|data: &u32, _env| data.to_string());
    let button = Button::new("Count", |_ctx, data, _env| *data += 1);

    let mut row = Row::new();
    row.add_child(Align::centered(label), 1.0);
    row.add_child(Align::centered(button), 1.0);
    row
}
