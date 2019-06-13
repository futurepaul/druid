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

//! A scrollable column widget

use std::any::Any;

use crate::widget::{ScrollEvent, Column, Widget};
use crate::{
    BoxConstraints, Geometry, HandlerCtx, Id, LayoutCtx, LayoutResult, MouseEvent, PaintCtx, Ui,
};

use kurbo::{Line, Rect};
use piet::{FillRule, RenderContext};

pub struct ScrollColumn {
    box_size: (f32, f32),
    scroll: f32,
}

impl ScrollColumn {
    pub fn new() -> ScrollColumn {
        ScrollColumn {
            box_size: (50.0, 50.0),
            scroll: 0.0,
        }
    }

    pub fn ui(self, children: &[Id], ctx: &mut Ui) -> Id {
        ctx.add(self, children)
    }
}

impl Widget for ScrollColumn {
    fn layout(
        &mut self,
        bc: &BoxConstraints,
        children: &[Id],
        _size: Option<(f32, f32)>,
        ctx: &mut LayoutCtx,
    ) -> LayoutResult {
        let mut y = 0.0;
        for child in children {
            ctx.position_child(*child, (y, 0.0));
            y += 20.0;
        }
        LayoutResult::Size(self.box_size)
    }

    fn scroll(&mut self, event: &ScrollEvent, ctx: &mut HandlerCtx) {
        dbg!("scrolled");
        if event.dy != 0.0 {
            self.scroll = (self.scroll + event.dy).max(0.0);
            // TODO: cap scroll past end; requires geometry, which should be
            // available from HandlerCtx, but this is not plumbed in druid.
            ctx.invalidate();
        }
    }

    fn mouse_moved(&mut self, x: f32, y: f32, ctx: &mut HandlerCtx) {
        dbg!("moused");
        self.scroll =  y;
        ctx.invalidate();
    }

}
