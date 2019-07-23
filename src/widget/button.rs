// Copyright 2018 The xi-editor Authors.
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

//! A button widget.

use std::marker::PhantomData;

use crate::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx,
    UpdateCtx, Widget, WidgetPod
};

use crate::kurbo::{Point, Size, Rect, Vec2, RoundedRect};
use crate::piet::{Color, FillRule, FontBuilder, Text, TextLayout, TextLayoutBuilder, Gradient, GradientStop, LinearGradient};
use crate::{Piet, RenderContext};

const BLACK: Color = Color::rgba32(0x00_00_00_ff);
const BLACK_ISH: Color = Color::rgba32(0x21_21_21_ff);
const BORDER: Color = Color::rgba32(0x3a_3a_3a_ff);
const BORDER_LIGHT: Color = Color::rgba32(0xa1_a1_a1_ff);

const LABEL_TEXT_COLOR: Color = Color::rgba32(0xf0_f0_ea_ff);
const FONT_SIZE: f64 = 14.0;

pub struct Label {
    text: String,
    size: Size,
}

pub struct Button<T: Data> {
    label: WidgetPod<T, Box<dyn Widget<T>>>,
}

pub struct DynLabel<T: Data, F: FnMut(&T, &Env) -> String> {
    label_closure: F,
    phantom: PhantomData<T>,
    size: Size
}

impl Label {
    /// Discussion question: should this return Label or a wrapped
    /// widget (with WidgetPod)?
    pub fn new(text: impl Into<String>) -> Label {
        Label {
            text: text.into(),
            size: Size::ZERO,
        }
    }

    fn get_layout(
        &self,
        text: &mut <Piet as RenderContext>::Text,
        font_size: f64,
    ) -> <Piet as RenderContext>::TextLayout {
        // TODO: caching of both the format and the layout
        let font = text
            .new_font_by_name("Roboto", font_size)
            .unwrap()
            .build()
            .unwrap();
        text.new_text_layout(&font, &self.text)
            .unwrap()
            .build()
            .unwrap()
    }
}

impl<T: Data> Widget<T> for Label {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, _base_state: &BaseState, _data: &T, _env: &Env) {
        let text = paint_ctx.render_ctx.text();
        let text_layout = self.get_layout(text, FONT_SIZE);
        let brush = paint_ctx.render_ctx.solid_brush(LABEL_TEXT_COLOR);

        paint_ctx
            .render_ctx
            .draw_text(&text_layout, Point::new(0., 0.8 * FONT_SIZE), &brush);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        _data: &T,
        _env: &Env,
    ) -> Size {
        if self.size == Size::ZERO {
            let text = layout_ctx.text();
            let text_layout = self.get_layout(text, FONT_SIZE);
            self.size = Size::new(text_layout.width(), FONT_SIZE);
        }

        self.size
    }

    fn event(
        &mut self,
        _event: &Event,
        _ctx: &mut EventCtx,
        _data: &mut T,
        _env: &Env,
    ) -> Option<Action> {
        None
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&T>, _data: &T, _env: &Env) {}
}

impl<T: Data> Button<T> {
    pub fn new(label: impl Widget<T> + 'static) -> Button<T> {
        Button {
            label: WidgetPod::new(label).boxed(),
        }
    }
}

impl<T: Data> Widget<T> for Button<T> {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        let is_active = base_state.is_active();
        let is_hot = base_state.is_hot();

        let rounded_rect = RoundedRect::from_origin_size(Point::ORIGIN, base_state.size().to_vec2(), 4.);
        
        let bg_gradient = if is_active {
            paint_ctx.render_ctx.gradient(Gradient::Linear(LinearGradient {
            start: rounded_rect.origin().to_vec2(),
            end: (rounded_rect.origin() + Vec2::new(0., base_state.size().height)).to_vec2(),
            stops: vec![GradientStop { pos: 0.0, color: BLACK}, GradientStop { pos: 1.0, color: BLACK_ISH}]
        })).unwrap()
        } else {
paint_ctx.render_ctx.gradient(Gradient::Linear(LinearGradient {
            start: rounded_rect.origin().to_vec2(),
            end: (rounded_rect.origin() + Vec2::new(0., base_state.size().height)).to_vec2(),
            stops: vec![GradientStop { pos: 0.0, color: BLACK_ISH}, GradientStop { pos: 1.0, color: BLACK}]
        })).unwrap()
        };

        let border_color = if is_hot {
            BORDER_LIGHT
        } else {
            BORDER
        };

        let outline_brush = paint_ctx.render_ctx.solid_brush(border_color);
        paint_ctx.render_ctx.stroke(rounded_rect, &outline_brush, 2.0, None);
        paint_ctx.render_ctx.fill(rounded_rect, &bg_gradient, FillRule::NonZero);
        self.label.paint_with_offset(paint_ctx, data, env);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let label_size = self.label.layout(layout_ctx, bc, data, env);
        let lr_pad = 14.;
        let tb_pad = 8.;
        self.label.set_layout_rect(Rect::from_origin_size(
            Point::new(lr_pad, tb_pad),
            label_size,
        ));
        let width = label_size.width + lr_pad * 2.;
        let height = label_size.height + tb_pad * 2.;
        Size::new(width, height)
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        _data: &mut T,
        _env: &Env,
    ) -> Option<Action> {
        let mut result = None;
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.invalidate();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.invalidate();
                    if ctx.is_hot() {
                        result = Some(Action::from_str("hit"));
                    }
                }
            }
            // TODO: don't handle this, handle HotChanged, when that's wired.
            Event::MouseMoved(_) => {
                ctx.invalidate();
            }
            _ => (),
        }
        result
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&T>, _data: &T, _env: &Env) {}
}

impl<T: Data, F: FnMut(&T, &Env) -> String> DynLabel<T, F> {
    pub fn new(label_closure: F) -> DynLabel<T, F> {
        DynLabel {
            label_closure,
            phantom: Default::default(),
            size: Size::ZERO
        }
    }

    fn get_layout(
        &mut self,
        text_ctx: &mut <Piet as RenderContext>::Text,
        font_size: f64,
        data: &T,
        env: &Env,
    ) -> <Piet as RenderContext>::TextLayout {
        let text = (self.label_closure)(data, env);
        // TODO: caching of both the format and the layout
        let font = text_ctx
            .new_font_by_name("Roboto", font_size)
            .unwrap()
            .build()
            .unwrap();
        text_ctx
            .new_text_layout(&font, &text)
            .unwrap()
            .build()
            .unwrap()
    }
}

impl<T: Data, F: FnMut(&T, &Env) -> String> Widget<T> for DynLabel<T, F> {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, _base_state: &BaseState, data: &T, env: &Env) {
        let text = paint_ctx.render_ctx.text();
        let text_layout = self.get_layout(text, FONT_SIZE, data, env);
        let brush = paint_ctx.render_ctx.solid_brush(LABEL_TEXT_COLOR);
        paint_ctx
            .render_ctx
            .draw_text(&text_layout, Point::new(0., 0.8 * FONT_SIZE), &brush);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        if self.size == Size::ZERO {
            let text = layout_ctx.text();
            let text_layout = self.get_layout(text, FONT_SIZE, data, env);
            self.size = Size::new(text_layout.width(), FONT_SIZE);
        }

        self.size
    }

    fn event(
        &mut self,
        _event: &Event,
        _ctx: &mut EventCtx,
        _data: &mut T,
        _env: &Env,
    ) -> Option<Action> {
        None
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&T>, _data: &T, _env: &Env) {
        self.size = Size::ZERO;
        ctx.invalidate();
    }
}
