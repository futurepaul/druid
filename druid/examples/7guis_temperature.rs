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

use druid::widget::{Adapt, Align, Label, NumericTextBox, Padding, Prism, Row, TextBox};
use druid::{AppLauncher, Data, Widget, WindowDesc};

#[derive(Clone, Data)]
struct AppState {
    temp_c: f64,
}

impl AppState {
    pub fn get_fahrenheit(&self) -> f64 {
        32.0 + (1.8 * self.temp_c)
    }

    pub fn set_celsius_from_fahrenheit(&mut self, new_value: f64) {
        self.temp_c = (new_value - 32.0) / 1.8;
    }
}

pub mod lenses {
    pub mod app_state {
        use super::super::AppState;
        use super::super::Lens2;
        use druid::Data;
        pub struct TempC;
        pub struct TempF;

        impl Lens2<AppState, f64> for TempC {
            fn get<V, F: FnOnce(&f64) -> V>(&self, data: &AppState, f: F) -> V {
                f(&data.temp_c)
            }

            fn with_mut<V, F: FnOnce(&mut f64) -> V>(&self, data: &mut AppState, f: F) -> V {
                f(&mut data.temp_c)
            }
        }

        impl Lens2<AppState, f64> for TempF {
            fn get<V, F: FnOnce(&f64) -> V>(&self, data: &AppState, f: F) -> V {
                f(&data.get_fahrenheit())
            }

            fn with_mut<V, F: FnOnce(&mut f64) -> V>(&self, data: &mut AppState, f: F) -> V {
                let mut state = data.get_fahrenheit();
                let save_state = state;
                let val = f(&mut state);
                if !state.same(&save_state) {
                    data.set_celsius_from_fahrenheit(state);
                };
                val
            }
        }
    }
}

fn main() {
    let main_window = WindowDesc::new(ui_builder);
    let state = AppState { temp_c: 0.0 };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(state)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppState> {
    let celsius_box = Lens2Wrap::new(NumericTextBox::new(), lenses::app_state::TempC);

    let fahrenheit_box = Lens2Wrap::new(NumericTextBox::new(), lenses::app_state::TempF);

    let celsius_label = Label::new("Celsius = ");
    let fahrenheit_label = Label::new("Fahrenheit");

    let mut row = Row::new();
    row.add_child(Align::centered(Padding::new(5.0, celsius_box)), 1.0);
    row.add_child(Align::centered(Padding::new(5.0, celsius_label)), 0.0);
    row.add_child(Align::centered(Padding::new(5.0, fahrenheit_box)), 1.0);
    row.add_child(Align::centered(Padding::new(5.0, fahrenheit_label)), 0.0);
    row
}

// LENS2 COPY PASTE

use std::marker::PhantomData;

use druid::kurbo::Size;

use druid::{BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx};

pub trait Lens2<T, U> {
    /// Get non-mut access to the field.
    ///
    /// Consider renaming, the signature suggests returning a reference, but
    /// it actually calls a closure with the field, allowing more flexibility
    /// in synthesizing it on demand.
    fn get<V, F: FnOnce(&U) -> V>(&self, data: &T, f: F) -> V;

    /// Get mutable access to the field.
    ///
    /// This method is defined in terms of a closure, rather than simply
    /// yielding a mutable reference, because it is intended to be used
    /// with value-type data (also known as immutable data structures).
    /// For example, a lens for an immutable list might be implemented by
    /// cloning the list, giving the closure mutable access to the clone,
    /// then updating the reference after the closure returns.
    fn with_mut<V, F: FnOnce(&mut U) -> V>(&self, data: &mut T, f: F) -> V;
}

// Discussion: it might be even better to make this a blanket impl for tuples.
pub struct Pair<L1, L2> {
    lens1: L1,
    lens2: L2,
}

impl<L1, L2> Pair<L1, L2> {
    pub fn new(lens1: L1, lens2: L2) -> Pair<L1, L2> {
        Pair { lens1, lens2 }
    }
}

impl<T: Data, U1: Data, U2: Data, L1: Lens2<T, U1>, L2: Lens2<T, U2>> Lens2<T, (U1, U2)>
    for Pair<L1, L2>
{
    fn get<V, F: FnOnce(&(U1, U2)) -> V>(&self, data: &T, f: F) -> V {
        self.lens1.get(data, |data1| {
            self.lens2.get(data, |data2| {
                let data = (data1.to_owned(), data2.to_owned());
                f(&data)
            })
        })
    }

    fn with_mut<V, F: FnOnce(&mut (U1, U2)) -> V>(&self, data: &mut T, f: F) -> V {
        let ((data1, data2), val, delta1, delta2) = self.lens1.get(data, |data1| {
            self.lens2.get(data, |data2| {
                let mut data = (data1.to_owned(), data2.to_owned());
                let val = f(&mut data);
                let delta1 = !data1.same(&data.0);
                let delta2 = !data2.same(&data.1);
                (data, val, delta1, delta2)
            })
        });
        if delta1 {
            self.lens1.with_mut(data, |d1| *d1 = data1);
        }
        if delta2 {
            self.lens2.with_mut(data, |d2| *d2 = data2);
        }
        val
    }
}

// A case can be made this should be in the `widget` module.

/// A wrapper for its widget subtree to have access to a part
/// of its parent's data.
///
/// Every widget in druid is instantiated with access to data of some
/// type; the root widget has access to the entire application data.
/// Often, a part of the widget hiearchy is only concerned with a part
/// of that data. The `Lens2Wrap` widget is a way to "focus" the data
/// reference down, for the subtree. One advantage is performance;
/// data changes that don't intersect the scope of the lens aren't
/// propagated.
///
/// Another advantage is generality and reuse. If a widget (or tree of
/// widgets) is designed to work with some chunk of data, then with a
/// lens that same code can easily be reused across all occurrences of
/// that chunk within the application state.
///
/// This wrapper takes a [`Lens2`] as an argument, which is a specification
/// of a struct field, or some other way of narrowing the scope.
///
/// [`Lens2`]: trait.Lens.html
pub struct Lens2Wrap<U, L, W> {
    inner: W,
    lens: L,
    // The following is a workaround for otherwise getting E0207.
    phantom: PhantomData<U>,
}

impl<U, L, W> Lens2Wrap<U, L, W> {
    /// Wrap a widget with a lens.
    ///
    /// When the lens has type `Lens2<T, U>`, the inner widget has data
    /// of type `U`, and the wrapped widget has data of type `T`.
    pub fn new(inner: W, lens: L) -> Lens2Wrap<U, L, W> {
        Lens2Wrap {
            inner,
            lens,
            phantom: Default::default(),
        }
    }
}

impl<T, U, L, W> Widget<T> for Lens2Wrap<U, L, W>
where
    T: Data,
    U: Data,
    L: Lens2<T, U>,
    W: Widget<U>,
{
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        let inner = &mut self.inner;
        self.lens
            .get(data, |data| inner.paint(paint_ctx, base_state, data, env));
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let inner = &mut self.inner;
        self.lens.get(data, |data| inner.layout(ctx, bc, data, env))
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let inner = &mut self.inner;
        self.lens
            .with_mut(data, |data| inner.event(ctx, event, data, env));
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&T>, data: &T, env: &Env) {
        let inner = &mut self.inner;
        let lens = &self.lens;
        if let Some(old_data) = old_data {
            lens.get(old_data, |old_data| {
                lens.get(data, |data| {
                    if !old_data.same(data) {
                        inner.update(ctx, Some(old_data), data, env);
                    }
                })
            })
        } else {
            lens.get(data, |data| inner.update(ctx, None, data, env));
        }
    }
}
