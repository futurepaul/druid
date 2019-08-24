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

use std::marker::PhantomData;

use druid::kurbo::Size;
use druid::piet::UnitPoint;
use druid::shell::{runloop, WindowBuilder};
use druid::widget::{
    ActionWrapper, Align, Button, CheckBox, Column, DynLabel, Padding, ProgressBar, Scroll, Slider,
    TextBox,
};
use druid::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx, UiMain,
    UiState, UpdateCtx, Widget,
};

//T is what the app gives us
//U is what the inner widget needs
//F converts from T to U
//G converts from U to T
pub struct DynWidget<T: Data, U: Data, F: FnMut(&T, &Env) -> U, G: FnMut(&U, &Env) -> T> {
    in_closure: F,
    out_closure: G,
    phantom: PhantomData<T>,
    widget: Box<dyn Widget<U>>,
}

impl<T: Data, U: Data, F: FnMut(&T, &Env) -> U, G: FnMut(&U, &Env) -> T> DynWidget<T, U, F, G> {
    pub fn new(
        widget: impl Widget<U> + 'static,
        in_closure: F,
        out_closure: G,
    ) -> DynWidget<T, U, F, G> {
        DynWidget {
            in_closure,
            out_closure,
            phantom: Default::default(),
            widget: Box::new(widget),
        }
    }
}

impl<T: Data, U: Data, F: FnMut(&T, &Env) -> U, G: FnMut(&U, &Env) -> T> Widget<T>
    for DynWidget<T, U, F, G>
{
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        let converted_data = (self.in_closure)(data, env);
        self.widget
            .paint(paint_ctx, base_state, &converted_data, env);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let converted_data = (self.in_closure)(data, env);
        self.widget.layout(layout_ctx, bc, &converted_data, env)
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut T,
        env: &Env,
    ) -> Option<Action> {
        let mut converted = (self.in_closure)(data, env);
        self.widget.event(event, ctx, &mut converted, env);
        *data = (self.out_closure)(&converted, env);
        None
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&T>, data: &T, env: &Env) {
        let converted_data = (self.in_closure)(data, env);
        self.widget.update(ctx, None, &converted_data, env);
    }
}

fn main() {
    druid_shell::init();

    let mut run_loop = runloop::RunLoop::new();
    let mut builder = WindowBuilder::new();

    let mut col = Column::new();
    let label_1 = DynLabel::new(|data: &f64, _env| format!("actual value: {0:.2}", data));
    let label_2 = DynLabel::new(|data: &f64, _env| format!("2x the value: {0:.2}", data * 2.0));
    let bar = ProgressBar::new();
    let slider = Slider::new();

    let button_1 = ActionWrapper::new(
        Align::centered(Button::shrink_to_fit("Shrink to fit")),
        move |data: &mut f64, _env| *data += 0.1,
    );
    let button_2 = ActionWrapper::new(
        Button::sized("Sized", 200.0, 100.0),
        move |data: &mut f64, _env| *data -= 0.1,
    );

    let button_3 = ActionWrapper::new(Button::new("Flex Basic"), move |data: &mut f64, _env| {
        *data += 0.05
    });

    let button_4 = ActionWrapper::new(
        Button::padded("Padded", 10., 5.),
        move |data: &mut f64, _env| *data -= 0.05,
    );

    let button_5 = ActionWrapper::new(
        Button::centered("Flex Centered"),
        move |data: &mut f64, _env| *data += 0.05,
    );

    let textbox = DynWidget::new(
        TextBox::new(),
        |input: &f64, _env| input.to_string(),
        |output: &String, _env| output.parse::<f64>().unwrap_or(0.),
    );

    let checkbox = DynWidget::new(
        CheckBox::new(),
        |input: &f64, _env| input.to_bits() == 1.0_f64.to_bits() || input > &1.0,
        |output: &bool, _env| {
            if *output {
                1.0
            } else {
                0.0
            }
        },
    );

    col.add_child(Padding::uniform(5.0, bar), 1.0);
    col.add_child(Padding::uniform(5.0, slider), 1.0);
    col.add_child(Padding::uniform(5.0, label_1), 1.0);
    col.add_child(Padding::uniform(5.0, label_2), 1.0);
    col.add_child(Padding::uniform(5.0, button_1), 1.0);
    col.add_child(Padding::uniform(5.0, button_2), 0.0);
    col.add_child(Padding::uniform(5.0, button_3), 1.0);
    col.add_child(
        Align::centered(Align::new(
            UnitPoint::new(0.9, 0.4),
            Padding::uniform(5.0, button_4),
        )),
        1.0,
    );
    col.add_child(Padding::uniform(5.0, button_5), 1.0);
    col.add_child(Padding::uniform(5.0, Align::centered(textbox)), 1.0);
    col.add_child(Padding::uniform(5.0, Align::centered(checkbox)), 0.0);

    let root = Align::centered(col);
    // let root = Align::centered(root);

    let state = UiState::new(Scroll::new(root), 0.7f64);
    builder.set_title("Widget demo");
    builder.set_handler(Box::new(UiMain::new(state)));
    let window = builder.build().unwrap();
    window.show();
    run_loop.run();
}
