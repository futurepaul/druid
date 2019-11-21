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
use druid::{AppLauncher, Widget, WindowDesc};

struct CtoCString;
struct CtoFString;

impl Prism<f64, String> for CtoCString {
    fn up<'a>(&self, data: &'a f64) -> String {
        format!("{:.*}", 2, data)
    }

    fn down(&self, data: String) -> Option<f64> {
        if let Ok(parsed) = data.parse::<f64>() {
            Some(parsed)
        } else {
            None
        }
    }
}

impl Prism<f64, String> for CtoFString {
    fn up<'a>(&self, data: &'a f64) -> String {
        format!("{:.*}", 2, data * 1.8 + 32.0)
    }

    fn down(&self, data: String) -> Option<f64> {
        if let Ok(parsed) = data.parse::<f64>() {
            let c = (parsed - 32.0) / 1.8;
            Some(c)
        } else {
            None
        }
    }
}

// fn get<'a>(&self, data: &'a #ty) -> &'a #field_ty {
//     &data.#field_name
// }

// fn with_mut<V, F: FnOnce(&mut #field_ty) -> V>(&self, data: &mut #ty, f: F) -> V {
//     f(&mut data.#field_name)
// }

fn main() {
    let main_window = WindowDesc::new(ui_builder);
    let temp_data_c = 0.0_f64;
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(temp_data_c)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<f64> {
    // let celsius_box = Adapt::new(TextBox::new(), CtoCString);
    let celsius_box = NumericTextBox::new();

    // let fahrenheit_box = Adapt::new(TextBox::new(), CtoFString);
    let fahrenheit_box = NumericTextBox::new();

    let celsius_label = Label::new("Celsius = ");
    let fahrenheit_label = Label::new("Fahrenheit");

    let mut row = Row::new();
    row.add_child(Align::centered(Padding::new(5.0, celsius_box)), 1.0);
    row.add_child(Align::centered(Padding::new(5.0, celsius_label)), 0.0);
    row.add_child(Align::centered(Padding::new(5.0, fahrenheit_box)), 1.0);
    row.add_child(Align::centered(Padding::new(5.0, fahrenheit_label)), 0.0);
    row
}
