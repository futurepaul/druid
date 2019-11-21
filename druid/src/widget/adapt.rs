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

use std::marker::PhantomData;

use crate::{
    BaseState, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx, Size, UpdateCtx,
    Widget,
};

pub trait Prism<T, U> {
    fn up<'a>(&self, data: &'a T) -> U;
    fn down(&self, data: U) -> Option<T>;
}

// fn get<'a>(&self, data: &'a T) -> &'a U;

// f_in: Box<dyn Fn(&T) -> U>,
// f_out: Box<dyn Fn(&mut U) -> T>,

// f_in: impl Fn(&T) -> U + 'static, f_out: impl Fn(&mut U) -> T + 'static,

/// A widget that accepts a closure to adapt data for its child
pub struct Adapt<U, P, W> {
    child: W,
    prism: P,
    phantom: PhantomData<U>,
}

impl<U, P, W> Adapt<U, P, W> {
    pub fn new(child: W, prism: P) -> Adapt<U, P, W> {
        Adapt {
            child,
            prism,
            phantom: Default::default(),
        }
    }
}

impl<T, U, P, W> Widget<T> for Adapt<U, P, W>
where
    T: Data,
    U: Data,
    P: Prism<T, U>,
    W: Widget<U>,
{
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        self.child
            .paint(paint_ctx, base_state, &self.prism.up(data), env);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        bc.debug_check("Adapt");

        self.child
            .layout(layout_ctx, &bc, &self.prism.up(data), env)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let mut adapted_data = self.prism.up(data).clone();

        if ctx.has_focus() {
            if let Some(down) = self.prism.down(adapted_data) {
                *data = down;
            };
        } else {
            let mut adapted_data = self.prism.up(data).clone();
            self.child.event(ctx, event, &mut adapted_data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&T>, data: &T, env: &Env) {
        if let Some(old_data) = old_data {
            if self.prism.up(old_data).same(&self.prism.up(data)) {
                return;
            }
        }

        // TODO: I lost a fight with the borrow checker
        if let Some(old) = old_data {
            self.child
                .update(ctx, Some(&self.prism.up(old)), &self.prism.up(data), env);
        } else {
            self.child.update(ctx, None, &self.prism.up(data), env);
        }
    }
}
