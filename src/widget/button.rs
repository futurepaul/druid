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
    RenderContext, UpdateCtx, Widget, WidgetPod,
};

use crate::kurbo::{Point, Rect, RoundedRect, Size, Vec2};
use crate::piet::{
    FontBuilder, LinearGradient, PietText, PietTextLayout, Text, TextLayout, TextLayoutBuilder,
    UnitPoint,
};
use crate::theme;

enum HorizontalAlignment {
    Leading,
    Center,
    Trailing,
}

enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

/// A label with static text.
pub struct Label {
    text: String,
    align: (HorizontalAlignment, VerticalAlignment),
}

/// A button with a static label.
pub struct Button<T: Data> {
    pub label: WidgetPod<T, Box<dyn Widget<T>>>,
    pub size: Option<Size>,
    pub padding: Option<(f64, f64)>,
}

/// A label with dynamic text.
///
/// The provided closure is called on update, and its return
/// value is used as the text for the label.
pub struct DynLabel<T: Data, F: FnMut(&T, &Env) -> String> {
    label_closure: F,
    phantom: PhantomData<T>,
}

impl Label {
    /// Discussion question: should this return Label or a wrapped
    /// widget (with WidgetPod)?
    pub fn new(text: impl Into<String>) -> Label {
        Label {
            text: text.into(),
            align: (HorizontalAlignment::Center, VerticalAlignment::Center),
        }
    }

    fn get_layout(&self, t: &mut PietText, font_name: &str, font_size: f64) -> PietTextLayout {
        // TODO: caching of both the format and the layout
        let font = t
            .new_font_by_name(font_name, font_size)
            .unwrap()
            .build()
            .unwrap();
        t.new_text_layout(&font, &self.text)
            .unwrap()
            .build()
            .unwrap()
    }
}

impl<T: Data> Widget<T> for Label {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, _data: &T, env: &Env) {
        let dbg_rect = Rect::from_origin_size(Point::ORIGIN, base_state.size());
        paint_ctx.fill(dbg_rect, &env.get(theme::SELECTION_COLOR));
        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL);
        let text_layout = self.get_layout(paint_ctx.text(), font_name, font_size);
        let (x, y) = match self.align {
            (HorizontalAlignment::Center, VerticalAlignment::Center) => {
                let x = (base_state.size().width / 2.) - (text_layout.width() / 2.);
                let y = (base_state.size().height / 2.) - (font_size * 1.2 / 2.);
                (x, y)
            }
            (HorizontalAlignment::Leading, VerticalAlignment::Top) => (0.0, 0.0),
            (HorizontalAlignment::Trailing, VerticalAlignment::Top) => {
                let x = base_state.size().width - text_layout.width();
                (x, 0.0)
            }
            _ => (0.0, 0.0),
        };
        paint_ctx.draw_text(
            &text_layout,
            (x, font_size + y),
            &env.get(theme::LABEL_COLOR),
        );
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &T,
        env: &Env,
    ) -> Size {
        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL);
        let text_layout = self.get_layout(layout_ctx.text, font_name, font_size);
        // This magical 1.2 constant helps center the text vertically in the rect it's given
        Size::new(text_layout.width(), font_size * 1.2)
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
    pub fn new(label: impl Into<String>) -> Button<T> {
        Button {
            label: WidgetPod::new(Label::new(label)).boxed(),
            size: None,
            padding: None,
        }
    }

    pub fn shrink_to_fit(label: impl Into<String>) -> Button<T> {
        Button {
            label: WidgetPod::new(Label::new(label)).boxed(),
            size: None,
            padding: Some((0.0, 0.0)),
        }
    }

    pub fn sized(label: impl Into<String>, width: f64, height: f64) -> Button<T> {
        Button {
            label: WidgetPod::new(Label::new(label)).boxed(),
            size: Some(Size::new(width, height)),
            padding: None,
        }
    }

    pub fn padded(label: impl Into<String>, hpad: f64, vpad: f64) -> Button<T> {
        Button {
            label: WidgetPod::new(Label::new(label)).boxed(),
            size: None,
            padding: Some((hpad, vpad)),
        }
    }
}

impl<T: Data> Widget<T> for Button<T> {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        let is_active = base_state.is_active();
        let is_hot = base_state.is_hot();

        let rounded_rect =
            RoundedRect::from_origin_size(Point::ORIGIN, base_state.size().to_vec2(), 4.);
        let bg_gradient = if is_active {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::BUTTON_LIGHT), env.get(theme::BUTTON_DARK)),
            )
        } else {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::BUTTON_DARK), env.get(theme::BUTTON_LIGHT)),
            )
        };

        let border_color = if is_hot {
            env.get(theme::BORDER_LIGHT)
        } else {
            env.get(theme::BORDER)
        };

        paint_ctx.stroke(rounded_rect, &border_color, 2.0);

        paint_ctx.fill(rounded_rect, &bg_gradient);

        self.label.paint_with_offset(paint_ctx, data, env);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let new_bc = BoxConstraints::new(Size::new(0.0, 0.0), bc.min());
        let label_size = self.label.layout(layout_ctx, &new_bc, data, env);
        if let Some(button_size) = self.size {
            let hpad = (button_size.width / 2.) - (label_size.width / 2.);
            let vpad = (button_size.height / 2.) - (label_size.height / 2.);
            self.label.set_layout_rect(Rect::from_origin_size(
                Point::ORIGIN + Vec2::new(hpad, vpad),
                label_size,
            ));
            return button_size;
        } else if let Some(padding) = self.padding {
            let hpad = padding.0;
            let vpad = padding.1;
            let padded_size = (label_size.to_vec2() + Vec2::new(hpad * 2., vpad * 2.)).to_size();
            self.label.set_layout_rect(Rect::from_origin_size(
                Point::ORIGIN + Vec2::new(hpad, vpad),
                label_size,
            ));
            return padded_size;
        } else {
            let button_size = bc.max();
            let hpad = (button_size.width / 2.) - (label_size.width / 2.);
            let vpad = (button_size.height / 2.) - (label_size.height / 2.);
            self.label.set_layout_rect(Rect::from_origin_size(
                Point::ORIGIN + Vec2::new(hpad, vpad),
                label_size,
            ));
            return bc.constrain(Size::new(label_size.width, label_size.height));
        }
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
            Event::HotChanged(_) => {
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
        }
    }

    fn get_layout(
        &mut self,
        rt: &mut PietText,
        font_name: &str,
        font_size: f64,
        data: &T,
        env: &Env,
    ) -> PietTextLayout {
        let text = (self.label_closure)(data, env);
        // TODO: caching of both the format and the layout
        let font = rt
            .new_font_by_name(font_name, font_size)
            .unwrap()
            .build()
            .unwrap();
        rt.new_text_layout(&font, &text).unwrap().build().unwrap()
    }
}

impl<T: Data, F: FnMut(&T, &Env) -> String> Widget<T> for DynLabel<T, F> {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, _base_state: &BaseState, data: &T, env: &Env) {
        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL);
        let text_layout = self.get_layout(paint_ctx.text(), font_name, font_size, data, env);
        paint_ctx.draw_text(&text_layout, (0., font_size), &env.get(theme::LABEL_COLOR));
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let font_name = env.get(theme::FONT_NAME);
        let font_size = env.get(theme::TEXT_SIZE_NORMAL);
        let text_layout = self.get_layout(layout_ctx.text, font_name, font_size, data, env);
        bc.constrain((text_layout.width(), 17.0))
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
        ctx.invalidate();
    }
}
