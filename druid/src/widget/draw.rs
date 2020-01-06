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

//! A widget for drawing.

use crate::kurbo::{BezPath, Point, Rect, RoundedRect, Size};
use crate::piet::{LineCap, LineJoin, LinearGradient, RenderContext, StrokeStyle, UnitPoint};
use crate::theme;
use crate::widget::Align;
use crate::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget, WidgetPod,
};

pub struct Draw<T: Data> {
    f: Box<dyn Fn(&Env, &mut PaintCtx, &T)>,
}

pub struct DrawWithChild<T: Data> {
    f: Box<dyn Fn(&Env, &mut PaintCtx, &T)>,
    child: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> Draw<T> {
    pub fn new(f: impl Fn(&Env, &mut PaintCtx, &T) + 'static) -> Draw<T> {
        Draw { f: Box::new(f) }
    }

    pub fn new_with_child(
        child: impl Widget<T> + 'static,
        f: impl Fn(&Env, &mut PaintCtx, &T) + 'static,
    ) -> DrawWithChild<T> {
        DrawWithChild {
            f: Box::new(f),
            child: WidgetPod::new(child).boxed(),
        }
    }
}

impl<T: Data> Widget<T> for Draw<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&T>, data: &T, env: &Env) {}

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        bc.debug_check("Draw");

        bc.max()
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        (self.f)(&env, paint_ctx, &data);
    }
}

impl<T: Data> Widget<T> for DrawWithChild<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.child.event(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&T>, data: &T, env: &Env) {
        self.child.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        bc.debug_check("DrawWithChild");

        let size = self.child.layout(layout_ctx, &bc, data, env);

        self.child
            .set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));

        size
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        (self.f)(&env, paint_ctx, &data);
        self.child.paint_with_offset(paint_ctx, data, env);
    }
}
