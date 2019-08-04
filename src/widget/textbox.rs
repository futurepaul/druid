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

use crate::{
    Action, BaseState, BoxConstraints, Cursor, Env, Event, EventCtx, KeyCode, LayoutCtx, PaintCtx,
    UpdateCtx, Widget,
};

use crate::kurbo::{Affine, Line, Point, RoundedRect, Size, Vec2};
use crate::piet::{
    Color, FillRule, FontBuilder, Piet, RenderContext, Text, TextLayout, TextLayoutBuilder,
};

use crate::unicode_segmentation::GraphemeCursor;

use druid_shell::clipboard::{ClipboardContext, ClipboardProvider};

const BACKGROUND_GREY_LIGHT: Color = Color::rgba32(0x3a_3a_3a_ff);
const BORDER_GREY: Color = Color::rgba32(0x5a_5a_5a_ff);
const PRIMARY_LIGHT: Color = Color::rgba32(0x5c_c4_ff_ff);
const PINK: Color = Color::rgba32(0xf3_00_21_ff);
const TEXT_COLOR: Color = Color::rgb24(0xf0_f0_ea);
const CURSOR_COLOR: Color = Color::WHITE;

const BOX_HEIGHT: f64 = 24.;
const FONT_SIZE: f64 = 14.0;
const BORDER_WIDTH: f64 = 1.;
const PADDING_TOP: f64 = 5.;
const PADDING_LEFT: f64 = 4.;

#[derive(Debug, Clone)]
pub struct TextBox {
    width: f64,
    cursor_pos: usize,
    hscroll_offset: f64,
    selected: bool,
}

impl TextBox {
    pub fn new(width: f64) -> TextBox {
        TextBox {
            width,
            cursor_pos: 3,
            hscroll_offset: 0.,
            selected: false,
        }
    }

    fn get_layout(
        &mut self,
        text: &mut <Piet as RenderContext>::Text,
        font_size: f64,
        data: &String,
    ) -> <Piet as RenderContext>::TextLayout {
        // TODO: caching of both the format and the layout
        let font = text
            .new_font_by_name("Segoe UI", font_size)
            .unwrap()
            .build()
            .unwrap();
        text.new_text_layout(&font, data).unwrap().build().unwrap()
    }

    fn insert_at_cursor(&mut self, src: &mut String, new: &str) {
        // TODO: handle incomplete graphemes
        let cursor = self.cursor_pos;
        src.replace_range(cursor..cursor, new);
        self.cursor_pos = cursor + new.len();
    }

    fn backspace(&mut self, src: &mut String) {
        let cursor = self.cursor_pos;
        self.prev_grapheme(&src);
        let new_cursor = self.cursor_pos;
        src.replace_range(new_cursor..cursor, "");
    }

    fn next_grapheme(&mut self, src: &str) {
        let mut c = GraphemeCursor::new(self.cursor_pos, src.len(), true);
        let next_boundary = c.next_boundary(src, 0).unwrap();
        if let Some(next) = next_boundary {
            self.cursor_pos = next;
        }
    }

    fn prev_grapheme(&mut self, src: &str) {
        let mut c = GraphemeCursor::new(self.cursor_pos, src.len(), true);
        let prev_boundary = c.prev_boundary(src, 0).unwrap();
        if let Some(prev) = prev_boundary {
            self.cursor_pos = prev;
        }
    }

    fn copy_text(&self, input: String) {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        //TODO: make this selection-aware
        ctx.set_contents(input).unwrap();
    }

    fn paste_text(&self) -> String {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        //TODO: make this selection-aware
        ctx.get_contents().unwrap_or("".to_string())
    }
}

impl Widget<String> for TextBox {
    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx,
        base_state: &BaseState,
        data: &String,
        _env: &Env,
    ) {
        let has_focus = base_state.has_focus();

        let border_color = if has_focus {
            PRIMARY_LIGHT
        } else {
            BORDER_GREY
        };

        // Paint the background
        let background_brush = paint_ctx.render_ctx.solid_brush(BACKGROUND_GREY_LIGHT);

        let clip_rect = RoundedRect::from_origin_size(
            Point::ORIGIN,
            Size::new(
                base_state.size().width - BORDER_WIDTH,
                base_state.size().height,
            )
            .to_vec2(),
            2.,
        );

        paint_ctx
            .render_ctx
            .fill(clip_rect, &background_brush, FillRule::NonZero);

        // Render text, selection, and cursor inside a clip
        paint_ctx
            .render_ctx
            .with_save(|rc| {
                rc.clip(clip_rect, FillRule::NonZero);

                // Layout and measure text
                let text = rc.text();
                let text_layout = self.get_layout(text, FONT_SIZE, data);
                let brush = rc.solid_brush(TEXT_COLOR);

                let text_height = FONT_SIZE * 0.8;
                let text_pos = Point::new(0.0 + PADDING_LEFT, text_height + PADDING_TOP);

                let max_text_width = text_layout.width();
                let mut cursor_x: f64 = 0.;

                // TODO: do hit testing instead of this substring hack!
                if let Some(substring) = data.get(..self.cursor_pos) {
                    cursor_x = self
                        .get_layout(rc.text(), FONT_SIZE, &substring.to_owned())
                        .width();
                }

                // If overflowing, shift the text
                let padded_width = self.width + (PADDING_LEFT * 2.);

                if max_text_width + (PADDING_LEFT * 2.) > self.width {
                    if cursor_x < self.width - (PADDING_LEFT * 2.) {
                        // Show head of text
                        self.hscroll_offset = 0.;
                    } else if cursor_x < self.hscroll_offset {
                        // Shift text so cursor is leftmost of box
                        self.hscroll_offset = cursor_x;
                    } else if cursor_x < max_text_width - padded_width {
                        // Shift text so cursor is rightmost of box
                        // TODO: This math isn't exactly right. I might need one more case?
                        self.hscroll_offset = cursor_x - padded_width;
                    } else {
                        // Show tail of text
                        self.hscroll_offset = (max_text_width - self.width) + (PADDING_LEFT * 2.);
                    }
                    rc.transform(Affine::translate(Vec2::new(-self.hscroll_offset, 0.)));
                }

                // Draw selection rect, also shifted
                if self.selected {
                    let selection_brush = rc.solid_brush(PINK);
                    let selection_pos = Point::new(PADDING_LEFT - 1., PADDING_TOP - 2.);
                    let selection_rect = RoundedRect::from_origin_size(
                        selection_pos,
                        Size::new(max_text_width + 2., FONT_SIZE + 4.).to_vec2(),
                        1.,
                    );
                    rc.fill(selection_rect, &selection_brush, FillRule::NonZero);
                }

                // Finally draw the text!
                rc.draw_text(&text_layout, text_pos, &brush);

                // Paint the cursor if focused
                if has_focus && !self.selected {
                    let brush = rc.solid_brush(CURSOR_COLOR);

                    let xy = text_pos + Vec2::new(cursor_x, 2. - FONT_SIZE);
                    let x2y2 = xy + Vec2::new(0., FONT_SIZE + 2.);
                    let line = Line::new(xy, x2y2);

                    rc.stroke(line, &brush, 1., None);
                }
                Ok(())
            })
            .unwrap();

        // Paint the border
        let border_brush = paint_ctx.render_ctx.solid_brush(border_color);

        paint_ctx
            .render_ctx
            .stroke(clip_rect, &border_brush, BORDER_WIDTH, None);
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        _bc: &BoxConstraints,
        _data: &String,
        _env: &Env,
    ) -> Size {
        Size::new(self.width, BOX_HEIGHT)
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut String,
        _env: &Env,
    ) -> Option<Action> {
        match event {
            Event::MouseDown(_) => {
                ctx.request_focus();
                self.selected = false;
                ctx.invalidate();
            }
            Event::MouseMoved(_) => {
                ctx.set_cursor(&Cursor::IBeam);
            }
            Event::KeyDown(key_event) => {
                match key_event {
                    event
                        if (event.mods.meta || event.mods.ctrl)
                            && (event.key_code == KeyCode::KeyC) =>
                    {
                        self.copy_text(data.to_string());
                    }
                    event
                        if (event.mods.meta || event.mods.ctrl)
                            && (event.key_code == KeyCode::KeyV) =>
                    {
                        let paste_text = self.paste_text();
                        if self.selected {
                            self.selected = false;
                            self.cursor_pos = paste_text.len();
                            *data = paste_text;
                        } else {
                            self.insert_at_cursor(data, &paste_text);
                        }
                    }
                    event
                        if (event.mods.meta || event.mods.ctrl)
                            && (event.key_code == KeyCode::KeyA) =>
                    {
                        self.selected = true;
                    }
                    event if event.key_code == KeyCode::Backspace => {
                        if self.selected {
                            self.selected = false;
                            *data = "".to_string();
                            self.cursor_pos = 0;
                        }

                        self.backspace(data);
                    }
                    event if event.key_code == KeyCode::ArrowLeft => {
                        if self.selected {
                            self.selected = false;
                            self.cursor_pos = 0;
                        } else {
                            self.prev_grapheme(data);
                        }
                    }
                    event if event.key_code == KeyCode::ArrowRight => {
                        if self.selected {
                            self.selected = false;
                            self.cursor_pos = data.len();
                        } else {
                            self.next_grapheme(data);
                        }
                    }
                    event if event.key_code.is_printable() => {
                        if self.selected {
                            self.selected = false;
                            *data = "".to_string();
                            self.cursor_pos = 0;
                        }
                        let incoming_text = event.text().unwrap_or("");
                        self.insert_at_cursor(data, incoming_text);
                    }
                    _ => {}
                }
                ctx.invalidate();
            }
            _ => (),
        }
        None
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: Option<&String>,
        _data: &String,
        _env: &Env,
    ) {
        ctx.invalidate();
    }
}
