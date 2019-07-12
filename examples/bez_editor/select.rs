//! The bezier pen tool.

use druid::kurbo::Point;
use druid::{Data, Event, MouseButton, MouseEvent};
use std::any::Any;

use super::{Contents, Mouse, Path, Tool, MIN_POINT_DISTANCE};

#[derive(Debug, Clone, PartialEq)]
struct Selection;

/// The state of the selection tool.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Select {
    mouse: Mouse,
    sel: Selection,
}

impl Select {
    pub fn new() -> Self {
        Select {
            mouse: Mouse::Up(Point::ZERO),
            sel: Selection,
        }
    }
}
//impl Data for Select {
//fn same(&self, other: &Self) -> bool {
//self.mouse == other.mouse && self.sel == other.sel
//}
//}

impl Tool for Select {
    fn event(&mut self, data: &mut Contents, event: &Event) -> bool {
        false
        //match event {
        //Event::MouseDown(mouse) if mouse.count == 1 => self.mouse_down(data, mouse),
        //Event::MouseDown(mouse) if mouse.count == 2 => self.double_click(data, mouse),
        //Event::MouseUp(mouse) => self.mouse_up(data, mouse),
        //Event::MouseMoved(mouse) => self.mouse_moved(data, mouse),
        //_ => false,
        //}
    }

    fn boxed_clone(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn same_impl(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Select>() {
            self.mouse == other.mouse && self.sel == other.sel
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        "select"
    }
}
