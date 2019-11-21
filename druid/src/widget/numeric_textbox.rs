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

//! A widget that adapts data into something its child can use

use crate::{
    BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, Size, UpdateCtx, Widget,
};

use crate::widget::TextBox;

pub struct NumericTextBox {
    child: Box<dyn Widget<String>>,
    cache: String,
}

impl NumericTextBox {
    pub fn new() -> NumericTextBox {
        NumericTextBox {
            child: Box::new(TextBox::new()),
            cache: "".to_string(),
        }
    }
}

impl Widget<f64> for NumericTextBox {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &f64, env: &Env) {
        let format = format!("{:.*}", 2, data);

        if base_state.has_focus() {
            self.child.paint(paint_ctx, base_state, &self.cache, env);
        } else {
            self.child.paint(paint_ctx, base_state, &format, env);
        }
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &f64,
        env: &Env,
    ) -> Size {
        bc.debug_check("NumericTextBox");

        let format = format!("{:.*}", 2, data);

        //TODO: this doesn't seem right but I don't know how to get has_focus
        self.child.layout(layout_ctx, &bc, &format, env)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f64, env: &Env) {
        let mut child_text;

        if ctx.has_focus() {
            child_text = self.cache.clone();
        } else {
            child_text = format!("{:.*}", 2, data);
        }

        self.child.event(ctx, event, &mut child_text, env);

        self.cache = child_text.clone();

        if let Ok(downcast) = child_text.parse::<f64>() {
            *data = downcast;
        };
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&f64>, data: &f64, env: &Env) {
        //TODO: feels like I'm missing some logic here but it works fine?
        if let Some(old) = old_data {
            self.child.update(
                ctx,
                Some(&format!("{:.*}", 2, old)),
                &format!("{:.*}", 2, data),
                env,
            );
        } else {
            self.child
                .update(ctx, None, &format!("{:.*}", 2, data), env);
        }
    }
}
