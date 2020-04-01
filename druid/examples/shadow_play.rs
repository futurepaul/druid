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

//! An example of a custom drawing widget.

use druid::kurbo::RoundedRect;
use druid::widget::prelude::*;
use druid::{Affine, AppLauncher, Color, LocalizedString, Point, Rect, WindowDesc};

struct CustomWidget;

impl Widget<String> for CustomWidget {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut String, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &String,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &String, _data: &String, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &String,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &String, _env: &Env) {
        let size = ctx.size();
        let rect = Rect::from_origin_size(Point::ORIGIN, size);

        let grey = &Color::rgb8(0xE0, 0xE5, 0xEC);
        let lighter_grey = &Color::rgba8(255, 255, 255, 127);
        let darker_grey = &Color::rgba8(0xA3, 0xB1, 0xC6, 127);
        ctx.fill(rect, grey);

        let rect = Rect::from_origin_size(
            ((size.width / 2.0) - 50.0, (size.height / 2.0) - 50.0),
            (100., 100.),
        );

        let rounded = RoundedRect::from_rect(rect, 10.0);

        ctx.with_save(move |ctx| {
            ctx.transform(Affine::translate((5.0, 5.0)));
            ctx.blurred_rect(rect, 10.0, darker_grey);
        });

        ctx.with_save(move |ctx| {
            ctx.transform(Affine::translate((-5.0, -5.0)));
            ctx.blurred_rect(rect, 10.0, lighter_grey);
        });

        ctx.fill(rounded, grey);
    }
}

fn main() {
    let window = WindowDesc::new(|| CustomWidget {}).title(LocalizedString::new("Neumorphism!"));
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch("I'm not using this but didn't bother to remove it".to_string())
        .expect("launch failed");
}
