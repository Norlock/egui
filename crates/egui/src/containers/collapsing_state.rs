use crate::*;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct WindowStoreState {
    /// Expand / collapse
    open: bool,

    /// Show / Hide (only used on egui::Window components)
    hidden: bool,

    /// Height of the region when open. Used for animations
    #[cfg_attr(feature = "serde", serde(default))]
    open_height: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct WidgetStoreState {
    /// Expand / collapse
    open: bool,

    /// Height of the region when open. Used for animations
    #[cfg_attr(feature = "serde", serde(default))]
    open_height: Option<f32>,
}

/// This is a a building block for building collapsing regions.
///
/// See [`CollapsingState::show_header`] for how to show a collapsing header with a custom header.
#[derive(Clone, Debug)]
pub struct WindowCollapsingState {
    id: Id,
    state: WindowStoreState,
}

/// This is a a building block for building collapsing regions.
///
/// See [`CollapsingState::show_header`] for how to show a collapsing header with a custom header.
pub struct WidgetCollapsingState {
    id: Id,
    state: WidgetStoreState,
}

pub trait CollapsingState {
    fn store(&self, ctx: &Context);
    fn remove(&self, ctx: &Context);
    fn id(&self) -> Id;
    fn is_open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    fn open_height(&mut self) -> &mut Option<f32>;

    fn toggle_open(&mut self, ui: &Ui) {
        self.set_open(!self.is_open());
        ui.ctx().request_repaint();
    }

    /// 0 for closed, 1 for open, with tweening
    fn openness(&self, ctx: &Context) -> f32 {
        if ctx.memory(|mem| mem.everything_is_visible()) {
            1.0
        } else {
            ctx.animate_bool(self.id(), self.is_open())
        }
    }

    fn show_header<HeaderRet>(
        mut self,
        ui: &mut Ui,
        add_header: impl FnOnce(&mut Ui) -> HeaderRet,
    ) -> HeaderResponse<'_, Self, HeaderRet>
    where
        Self: Sized,
    {
        //CommonCollapse::show_header(*self, ui, add_header)
        let header_response = ui.horizontal(|ui| {
            let prev_item_spacing = ui.spacing_mut().item_spacing;
            ui.spacing_mut().item_spacing.x = 0.0; // the toggler button uses the full indent width
            let collapser = self.show_default_button_indented(ui);
            ui.spacing_mut().item_spacing = prev_item_spacing;
            (collapser, add_header(ui))
        });

        HeaderResponse {
            state: self,
            ui,
            toggle_button_response: header_response.inner.0,
            header_response: InnerResponse {
                response: header_response.response,
                inner: header_response.inner.1,
            },
        }
    }

    /// Will toggle when clicked, etc.
    fn show_default_button_indented(&mut self, ui: &mut Ui) -> Response
    where
        Self: Sized,
    {
        CommonCollapse::show_button_indented(self, ui, CommonCollapse::paint_default_icon)
    }

    /// Will toggle when clicked, etc.
    fn show_button_indented(
        coll: &mut impl CollapsingState,
        ui: &mut Ui,
        icon_fn: impl FnOnce(&mut Ui, f32, &Response) + 'static,
    ) -> Response {
        let size = vec2(ui.spacing().indent, ui.spacing().icon_width);
        let (_id, rect) = ui.allocate_space(size);
        let response = ui.interact(rect, coll.id(), Sense::click());
        if response.clicked() {
            coll.toggle_open(ui);
        }

        let (mut icon_rect, _) = ui.spacing().icon_rectangles(response.rect);
        icon_rect.set_center(pos2(
            response.rect.left() + ui.spacing().indent / 2.0,
            response.rect.center().y,
        ));
        let openness = coll.openness(ui.ctx());
        let small_icon_response = response.clone().with_new_rect(icon_rect);
        icon_fn(ui, openness, &small_icon_response);
        response
    }
}

impl CollapsingState for WindowCollapsingState {
    fn store(&self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_persisted(self.id, self.state));
    }

    fn remove(&self, ctx: &Context) {
        ctx.data_mut(|d| d.remove::<WindowStoreState>(self.id));
    }

    fn id(&self) -> Id {
        self.id
    }

    fn is_open(&self) -> bool {
        self.state.open
    }

    fn set_open(&mut self, open: bool) {
        self.state.open = open;
    }

    fn open_height(&mut self) -> &mut Option<f32> {
        &mut self.state.open_height
    }
}

impl CollapsingState for WidgetCollapsingState {
    fn store(&self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_persisted(self.id, self.state));
    }

    fn remove(&self, ctx: &Context) {
        ctx.data_mut(|d| d.remove::<WidgetStoreState>(self.id));
    }

    fn id(&self) -> Id {
        self.id
    }

    fn is_open(&self) -> bool {
        self.state.open
    }

    fn set_open(&mut self, open: bool) {
        self.state.open = open;
    }

    fn open_height(&mut self) -> &mut Option<f32> {
        &mut self.state.open_height
    }
}

impl WindowCollapsingState {
    pub fn is_hidden(&self) -> bool {
        self.state.hidden
    }

    pub fn toggle_hidden(&mut self) {
        self.state.hidden = !self.state.hidden;
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.state.hidden = hidden;
    }

    pub fn load(ctx: &Context, id: Id, default_open: bool) -> Self {
        ctx.data_mut(|d| {
            d.get_persisted::<WindowStoreState>(id)
                .map(|state| Self { id, state })
        })
        .unwrap_or(WindowCollapsingState {
            id,
            state: WindowStoreState {
                open: default_open,
                hidden: false,
                open_height: None,
            },
        })
    }
}

impl WidgetCollapsingState {
    pub fn load(ctx: &Context, id: Id, default_open: bool) -> Self {
        ctx.data_mut(|d| {
            d.get_persisted::<WidgetStoreState>(id)
                .map(|state| Self { id, state })
        })
        .unwrap_or(WidgetCollapsingState {
            id,
            state: WidgetStoreState {
                open: default_open,
                open_height: None,
            },
        })
    }
}

pub struct CommonCollapse;

impl CommonCollapse {
    /// Will toggle when clicked, etc.
    pub(crate) fn show_default_button_with_size<Coll: CollapsingState>(
        coll: &mut Coll,
        ui: &mut Ui,
        button_size: Vec2,
    ) -> Response {
        let rect = ui.allocate_space(button_size).1;
        let response = ui.interact(rect, coll.id(), Sense::click());
        if response.clicked() {
            coll.toggle_open(ui);
        }
        let openness = coll.openness(ui.ctx());
        Self::paint_default_icon(ui, openness, &response);
        response
    }

    /// Will toggle when clicked, etc.
    pub fn show_default_button_indented(coll: &mut impl CollapsingState, ui: &mut Ui) -> Response {
        Self::show_button_indented(coll, ui, Self::paint_default_icon)
    }

    /// Will toggle when clicked, etc.
    pub fn show_button_indented<Coll: CollapsingState>(
        coll: &mut Coll,
        ui: &mut Ui,
        icon_fn: impl FnOnce(&mut Ui, f32, &Response) + 'static,
    ) -> Response {
        let size = vec2(ui.spacing().indent, ui.spacing().icon_width);
        let (_id, rect) = ui.allocate_space(size);
        let response = ui.interact(rect, coll.id(), Sense::click());
        if response.clicked() {
            coll.toggle_open(ui);
        }

        let (mut icon_rect, _) = ui.spacing().icon_rectangles(response.rect);
        icon_rect.set_center(pos2(
            response.rect.left() + ui.spacing().indent / 2.0,
            response.rect.center().y,
        ));
        let openness = coll.openness(ui.ctx());
        let small_icon_response = response.clone().with_new_rect(icon_rect);
        icon_fn(ui, openness, &small_icon_response);
        response
    }

    /// Show body if we are open, with a nice animation between closed and open.
    /// Indent the body to show it belongs to the header.
    ///
    /// Will also store the state.
    pub fn show_body_indented<Coll: CollapsingState, R>(
        coll: &mut Coll,
        header_response: &Response,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let id = coll.id();
        Self::show_body_unindented(coll, ui, |ui| {
            ui.indent(id, |ui| {
                // make as wide as the header:
                ui.expand_to_include_x(header_response.rect.right());
                add_body(ui)
            })
            .inner
        })
    }

    /// Show body if we are open, with a nice animation between closed and open.
    /// Will also store the state.
    pub fn show_body_unindented<T: CollapsingState, R>(
        component: &mut T,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let openness = component.openness(ui.ctx());

        if openness <= 0.0 {
            component.store(ui.ctx()); // we store any earlier toggling as promised in the docstring
            None
        } else if openness < 1.0 {
            Some(ui.scope(|child_ui| {
                let is_open = component.is_open();
                let open_height = component.open_height();

                let max_height = if is_open && open_height.is_none() {
                    // First frame of expansion.
                    // We don't know full height yet, but we will next frame.
                    // Just use a placeholder value that shows some movement:
                    10.0
                } else {
                    let full_height = open_height.unwrap_or_default();
                    remap_clamp(openness, 0.0..=1.0, 0.0..=full_height)
                };

                let mut clip_rect = child_ui.clip_rect();
                clip_rect.max.y = clip_rect.max.y.min(child_ui.max_rect().top() + max_height);
                child_ui.set_clip_rect(clip_rect);

                let ret = add_body(child_ui);

                let mut min_rect = child_ui.min_rect();
                *open_height = Some(min_rect.height());
                component.store(child_ui.ctx()); // remember the height

                // Pretend children took up at most `max_height` space:
                min_rect.max.y = min_rect.max.y.at_most(min_rect.top() + max_height);
                child_ui.force_set_min_rect(min_rect);
                ret
            }))
        } else {
            let ret_response = ui.scope(add_body);
            let full_size = ret_response.response.rect.size();
            let open_height = component.open_height();
            *open_height = Some(full_size.y);
            component.store(ui.ctx()); // remember the height
            Some(ret_response)
        }
    }

    /// Paint this [CollapsingState](CollapsingState)'s toggle button. Takes an [IconPainter](IconPainter) as the icon.
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// fn circle_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    ///     let stroke = ui.style().interact(&response).fg_stroke;
    ///     let radius = egui::lerp(2.0..=3.0, openness);
    ///     ui.painter().circle_filled(response.rect.center(), radius, stroke.color);
    /// }
    ///
    /// let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
    ///     ui.ctx(),
    ///     ui.make_persistent_id("my_collapsing_state"),
    ///     false,
    /// );
    ///
    /// let header_res = ui.horizontal(|ui| {
    ///     ui.label("Header");
    ///     state.show_toggle_button(ui, circle_icon);
    /// });
    ///
    /// state.show_body_indented(&header_res.response, ui, |ui| ui.label("Body"));
    /// # });
    /// ```
    pub fn show_toggle_button(
        component: &mut impl CollapsingState,
        ui: &mut Ui,
        icon_fn: impl FnOnce(&mut Ui, f32, &Response) + 'static,
    ) -> Response {
        Self::show_button_indented(component, ui, icon_fn)
    }

    /// Paint the arrow icon that indicated if the region is open or not
    pub fn paint_default_icon(ui: &mut Ui, openness: f32, response: &Response) {
        let visuals = ui.style().interact(response);

        let rect = response.rect;

        // Draw a pointy triangle arrow:
        let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
        let rect = rect.expand(visuals.expansion);
        let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
        use std::f32::consts::TAU;
        let rotation = emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
        for p in &mut points {
            *p = rect.center() + rotation * (*p - rect.center());
        }

        ui.painter().add(Shape::convex_polygon(
            points,
            visuals.fg_stroke.color,
            Stroke::NONE,
        ));
    }
}

#[must_use = "Remember to show the body"]
pub struct HeaderResponse<'ui, T: CollapsingState, HeaderRet> {
    state: T,
    ui: &'ui mut Ui,
    toggle_button_response: Response,
    header_response: InnerResponse<HeaderRet>,
}

impl<'ui, T: CollapsingState, HeaderRet> HeaderResponse<'ui, T, HeaderRet> {
    /// Returns the response of the collapsing button, the custom header, and the custom body.
    pub fn body<BodyRet>(
        mut self,
        add_body: impl FnOnce(&mut Ui) -> BodyRet,
    ) -> (
        Response,
        InnerResponse<HeaderRet>,
        Option<InnerResponse<BodyRet>>,
    ) {
        let body_response = CommonCollapse::show_body_indented(
            &mut self.state,
            &self.header_response.response,
            self.ui,
            add_body,
        );
        (
            self.toggle_button_response,
            self.header_response,
            body_response,
        )
    }

    /// Returns the response of the collapsing button, the custom header, and the custom body, without indentation.
    pub fn body_unindented<BodyRet>(
        mut self,
        add_body: impl FnOnce(&mut Ui) -> BodyRet,
    ) -> (
        Response,
        InnerResponse<HeaderRet>,
        Option<InnerResponse<BodyRet>>,
    ) {
        let body_response =
            CommonCollapse::show_body_unindented(&mut self.state, self.ui, add_body);
        (
            self.toggle_button_response,
            self.header_response,
            body_response,
        )
    }
}
