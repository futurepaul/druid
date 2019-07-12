//! The toolbar.

use std::sync::Arc;

use druid::kurbo::{Affine, BezPath, Line, Point, Rect, Shape, Size};
use druid::piet::{Color, FillRule, RenderContext};
use druid::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, KeyEvent, LayoutCtx, PaintCtx,
    UpdateCtx, Widget,
};

#[derive(Debug, Default, Clone)]
pub struct Toolbar {
    hot: Option<usize>,
    has_mouse: bool,
}

#[derive(Debug, Clone)]
pub struct ToolbarState {
    pub items: Arc<Vec<ToolbarItem>>,
    selected: usize,
}

#[derive(Debug, Clone)]
pub struct ToolbarItem {
    pub name: String,
    pub hotkey: Option<String>,
    icon: BezPath,
}

impl Data for ToolbarState {
    fn same(&self, other: &Self) -> bool {
        self.items.same(&other.items) && self.selected == other.selected
    }
}

impl ToolbarItem {
    pub fn new(name: impl Into<String>, hotkey: Option<impl Into<String>>, icon: BezPath) -> Self {
        let padding = TOOLBAR_ICON_PADDING * 2.;
        let icon = scale_path(
            &icon,
            (TOOLBAR_ITEM_WIDTH - padding, TOOLBAR_HEIGHT - padding),
        );
        ToolbarItem {
            name: name.into(),
            hotkey: hotkey.map(Into::into),
            icon,
        }
    }

    fn select() -> Self {
        let mut path = BezPath::new();
        path.move_to((45., 100.));
        path.line_to((55., 100.));
        path.line_to((55., 70.));
        path.line_to((80., 70.));
        path.line_to((50., 10.));
        path.line_to((20., 70.));
        path.line_to((45., 70.));
        path.close_path();
        path.apply_affine(Affine::rotate(-0.5));
        ToolbarItem::new("select", Some("v"), path)
    }

    fn pen() -> Self {
        let mut path = BezPath::new();
        path.move_to((173., 0.));
        path.line_to((277., 0.));
        path.line_to((277., 93.));
        path.curve_to((277., 93.), (364., 186.), (364., 265.));
        path.curve_to((364., 344.), (255., 481.), (255., 481.));
        path.curve_to((255., 481.), (86., 344.), (86., 265.));
        path.curve_to((86., 186.), (173., 93.), (173., 93.));
        path.close_path();
        path.apply_affine(Affine::rotate(-3.5));
        ToolbarItem::new("pen", Some("p"), path)
    }
}

impl ToolbarState {
    pub fn basic() -> Self {
        ToolbarState::new(vec![ToolbarItem::select(), ToolbarItem::pen()])
    }

    fn new(items: Vec<ToolbarItem>) -> Self {
        ToolbarState {
            items: Arc::new(items),
            selected: 0,
        }
    }

    /// Returns the item with a hotkey corresponding to `key`, if one exists.
    pub fn idx_for_key(&self, key: &KeyEvent) -> Option<usize> {
        self.items
            .iter()
            .position(|item| item.hotkey.as_ref().map(String::as_str) == key.text())
    }

    /// Set the selected tool, by index.
    pub fn set_selected(&mut self, selected: usize) {
        if selected < self.items.len() {
            self.selected = selected
        }
    }

    pub fn selected_item(&self) -> &ToolbarItem {
        &self.items[self.selected]
    }

    pub fn selected_idx(&self) -> usize {
        self.selected
    }
}

impl Toolbar {
    fn size(&self, state: &ToolbarState) -> Size {
        let width = state.items.len() as f64 * TOOLBAR_ITEM_WIDTH;
        Size::new(width, TOOLBAR_HEIGHT)
    }

    fn tool_at_pos(&self, state: &ToolbarState, pos: Point) -> Option<usize> {
        let Size { width, height } = self.size(state);
        if pos.x > 0. && pos.y > 0. && pos.x < width && pos.y < height {
            let idx = (pos.x / TOOLBAR_ITEM_WIDTH).trunc() as usize;
            Some(idx)
        } else {
            None
        }
    }
}

const TOOLBAR_HEIGHT: f64 = 32.;
const TOOLBAR_ICON_PADDING: f64 = 4.;
const TOOLBAR_ITEM_WIDTH: f64 = 32.;
const BG_COLOR: Color = Color::rgb24(0xca_ca_ca);
const BG_SELECTED_COLOR: Color = Color::rgb24(0x1C_6A_FF);
const BG_HOVER_COLOR: Color = Color::rgb24(0x8e_8e_8e);
const ICON_COLOR: Color = Color::rgb24(0x3e_3a_38);
const ICON_SELECTED_COLOR: Color = Color::rgb24(0xde_da_d8);

impl Widget<ToolbarState> for Toolbar {
    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx,
        _base_state: &BaseState,
        data: &ToolbarState,
        _env: &Env,
    ) {
        let bg_brush = paint_ctx.render_ctx.solid_brush(BG_COLOR);
        let bg_selected = paint_ctx.render_ctx.solid_brush(BG_SELECTED_COLOR);
        let bg_hover = paint_ctx.render_ctx.solid_brush(BG_HOVER_COLOR);
        let item_brush = paint_ctx.render_ctx.solid_brush(ICON_COLOR);
        let item_selected_brush = paint_ctx.render_ctx.solid_brush(ICON_SELECTED_COLOR);
        let item_size = Size::new(TOOLBAR_ITEM_WIDTH, TOOLBAR_HEIGHT);

        // the trailing edge of the last drawn button, for drawing separators if needed.
        let mut last = None;

        for (i, tool) in data.items.iter().enumerate() {
            let bg = if i == data.selected {
                &bg_selected
            } else if Some(i) == self.hot {
                &bg_hover
            } else {
                &bg_brush
            };

            let fg = if i == data.selected {
                &item_selected_brush
            } else {
                &item_brush
            };

            let item_rect = Rect::from_origin_size((i as f64 * TOOLBAR_ITEM_WIDTH, 0.), item_size);
            paint_ctx.render_ctx.fill(item_rect, &bg, FillRule::NonZero);

            let icon_size = tool.icon.bounding_box().size();
            let x_pad = TOOLBAR_ICON_PADDING.max((TOOLBAR_ITEM_WIDTH - icon_size.width) * 0.5);
            let y_pad = TOOLBAR_ICON_PADDING.max((TOOLBAR_HEIGHT - icon_size.height) * 0.5);
            let tool_pos = Affine::translate((x_pad + i as f64 * TOOLBAR_ITEM_WIDTH, y_pad));
            paint_ctx
                .render_ctx
                .fill(tool_pos * &tool.icon, &fg, FillRule::NonZero);

            if let Some(last) = last {
                let line = Line::new((last, 0.), (last, TOOLBAR_HEIGHT));
                paint_ctx.render_ctx.stroke(line, &item_brush, 0.5, None);
            }

            last = Some((i + 1) as f64 * TOOLBAR_ITEM_WIDTH);
        }
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &ToolbarState,
        _env: &Env,
    ) -> Size {
        bc.constrain(self.size(data))
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut ToolbarState,
        _env: &Env,
    ) -> Option<Action> {
        match event {
            Event::MouseDown(mouse) => {
                if let Some(_) = self.tool_at_pos(data, mouse.pos) {
                    self.has_mouse = true;
                    ctx.set_handled();
                    ctx.invalidate();
                }
            }

            Event::MouseUp(mouse) => {
                if self.has_mouse {
                    self.has_mouse = false;
                    ctx.set_handled();
                    ctx.invalidate();
                    if let Some(idx) = self.tool_at_pos(data, mouse.pos) {
                        data.selected = idx;
                        return Some(Action::from_str(data.items[idx].name.as_str()));
                    }
                }
            }

            Event::MouseMoved(mouse) => {
                let hot = self.tool_at_pos(data, mouse.pos);
                if hot != self.hot {
                    self.hot = hot;
                    ctx.set_active(self.has_mouse | hot.is_some());
                    ctx.invalidate();
                    ctx.set_handled();
                }
            }
            _ => (),
        }
        None
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: Option<&ToolbarState>,
        _data: &ToolbarState,
        _env: &Env,
    ) {
    }
}

fn scale_path(path: &BezPath, fitting_size: impl Into<Size>) -> BezPath {
    let mut out = path.clone();
    let fitting_size = fitting_size.into();
    let path_size = path.bounding_box().size();
    let scale_factor =
        (fitting_size.width / path_size.width).min(fitting_size.height / path_size.height);
    out.apply_affine(Affine::scale(scale_factor));
    let translation = Point::ZERO - out.bounding_box().origin();
    out.apply_affine(Affine::translate(translation));
    out
}
