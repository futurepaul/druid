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

//! A textbox widget.

use crate::widget::Widget;
use crate::{
  BoxConstraints, Geometry, HandlerCtx, Id, KeyEvent, KeyVariant, LayoutCtx, LayoutResult,
  MouseEvent, PaintCtx, Ui,
};

use kurbo::{Line, Rect};
use piet::{FillRule, Font, FontBuilder, RenderContext, Text, TextLayout, TextLayoutBuilder};
use piet_common::{CairoFont, Piet};

const BOX_HEIGHT: f32 = 24.;

pub struct TextBox {
  text: String,
  width: f64,
  font: Option<<<Piet<'static> as RenderContext>::Text as Text>::Font>,
}

impl TextBox {
  pub fn new(default_text: Option<String>, width: f64) -> TextBox {
    TextBox {
      text: default_text.unwrap_or_else(|| String::new()),
      width,
      font: None,
    }
  }
  pub fn ui(self, ctx: &mut Ui) -> Id {
    ctx.add(self, &[])
  }

  fn load_font(&mut self, rt: &mut Piet, font_size: f32) {
    let font = rt
      .text()
      .new_font_by_name("Segoe UI", font_size)
      .unwrap()
      .build()
      .unwrap();

    self.font = Some(font);
  }

  fn get_layout(&mut self, rt: &mut Piet, font_size: f32) -> <Piet as RenderContext>::TextLayout {
    // TODO: caching of both the format and the layout
    match &self.font {
      Some(font) => {
        return rt
          .text()
          .new_text_layout(&font, &self.text)
          .unwrap()
          .build()
          .unwrap()
      }
      _ => {
        self.load_font(rt, font_size);

        //QUESTION this recursion makes me uncomfortable
        //but it solved my borrowing issues!
        return self.get_layout(rt, font_size);
      }
    };
  }
}

impl Widget for TextBox {
  fn paint(&mut self, paint_ctx: &mut PaintCtx, geom: &Geometry) {
    let border_color = if paint_ctx.is_focused() {
      // Create active color
      0xff_00_00_ff
    } else {
      // Create inactive color
      0x55_55_55_ff
    };

    let brush = paint_ctx.render_ctx.solid_brush(border_color).unwrap();

    // Paint the border
    let (x, y) = geom.pos;
    let (width, height) = geom.size;
    let rect = Rect::new(
      x as f64,
      y as f64,
      x as f64 + width as f64,
      y as f64 + height as f64,
    );
    paint_ctx.render_ctx.stroke(rect, &brush, 1., None);

    // Paint the text
    let font_size = BOX_HEIGHT - 4.;
    let text_layout = self.get_layout(paint_ctx.render_ctx, font_size);
    let brush = paint_ctx.render_ctx.solid_brush(0xf0f0eaff).unwrap();

    let pos = (geom.pos.0, geom.pos.1 + font_size);
    paint_ctx.render_ctx.draw_text(&text_layout, pos, &brush);

    // Paint the cursor if focused
    if paint_ctx.is_focused() {
      let brush = paint_ctx.render_ctx.solid_brush(0xffffffff).unwrap();

      let (x, y) = (
        geom.pos.0 + text_layout.width() as f32 + 2.,
        geom.pos.1 + 2.,
      );

      let line = Line::new(
        (x as f64, y as f64),
        (x as f64, y as f64 + font_size as f64),
      );

      paint_ctx.render_ctx.stroke(line, &brush, 1., None);
    };
  }

  fn layout(
    &mut self,
    bc: &BoxConstraints,
    _children: &[Id],
    _size: Option<(f32, f32)>,
    _ctx: &mut LayoutCtx,
  ) -> LayoutResult {
    LayoutResult::Size(bc.constrain((self.width as f32, BOX_HEIGHT)))
  }

  fn mouse(&mut self, event: &MouseEvent, ctx: &mut HandlerCtx) -> bool {
    if event.count > 0 {
      ctx.set_focused(true);
      ctx.invalidate();
    }
    true
  }
  fn key(&mut self, event: &KeyEvent, ctx: &mut HandlerCtx) -> bool {
    //match on key event
    match event.key {
      KeyVariant::Char(ch) => {
        if ch == '\u{7f}' {
          self.text.pop();
        } else {
          self.text.push(ch);
        }
      }
      KeyVariant::Vkey(vk) => match vk {
        VK_BACK => {
          self.text.pop();
        }
        _ => {}
      },
    }
    // update the text string
    // call invalidate

    ctx.invalidate();
    true
  }

//When we receive a mouseclick we give the focus to our text box
//which gives us key inputs, which lets us update our state
//and then we invalidate our state and...