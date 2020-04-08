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

use druid::widget::{Label, List, Painter, Scroll};
use druid::{
    theme, AppLauncher, Data, Env, Lens, LocalizedString, RenderContext, Widget, WidgetExt,
    WindowDesc,
};

use std::sync::Arc;

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Too long list!");

#[derive(Clone, Data, Lens)]
struct AppState {
    items: Arc<Vec<String>>,
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    let items: Vec<String> = (1..3000)
        .collect::<Vec<u32>>()
        .iter()
        .map(|i| i.to_string())
        .collect();

    // create the initial app state
    let initial_state = AppState {
        items: Arc::new(items),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppState> {
    Scroll::new(
        List::new(|| {
            let painter = Painter::new(|ctx, _, env| {
                let bounds = ctx.size().to_rect();

                if ctx.is_hot() {
                    ctx.fill(bounds, &env.get(theme::PRIMARY_DARK));
                }

                if ctx.is_active() {
                    ctx.fill(bounds, &env.get(theme::PRIMARY_LIGHT));
                }
            });

            Label::new(|data: &String, _: &Env| data.clone())
                .padding(5.0)
                .background(painter)
                .on_click(|_, _, _| {
                    println!("hey");
                })
                .expand_width()
        })
        .lens(AppState::items),
    )
    .vertical()
}
