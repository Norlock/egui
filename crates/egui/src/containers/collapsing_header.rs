use crate::collapsing_state::*;
use crate::*;
use std::hash::Hash;

/// A event to expand / collapse the widget automatically
#[derive(PartialEq, Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum WidgetDisplayEvent {
    #[default]
    Expand,
    Collapse,
    ToggleCollapse,
}

/// A function that paints an icon indicating if the region is open or not
pub type IconPainter = Box<dyn FnOnce(&mut Ui, f32, &Response)>;

/// A header which can be collapsed/expanded, revealing a contained [`Ui`] region.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// egui::CollapsingHeader::new("Heading")
///     .show(ui, |ui| {
///         ui.label("Body");
///     });
///
/// // Short version:
/// ui.collapsing("Heading", |ui| { ui.label("Body"); });
/// # });
/// ```
///
/// If you want to customize the header contents, see [`CollapsingState::show_header`].
#[must_use = "You should call .show()"]
pub struct CollapsingHeader {
    text: WidgetText,
    default_open: bool,
    open: Option<bool>,
    display_event: Option<WidgetDisplayEvent>,
    id_source: Id,
    enabled: bool,
    selectable: bool,
    selected: bool,
    show_background: bool,
    icon: Option<IconPainter>,
}

impl CollapsingHeader {
    /// The [`CollapsingHeader`] starts out collapsed unless you call `default_open`.
    ///
    /// The label is used as an [`Id`] source.
    /// If the label is unique and static this is fine,
    /// but if it changes or there are several [`CollapsingHeader`] with the same title
    /// you need to provide a unique id source with [`Self::id_source`].
    pub fn new(text: impl Into<WidgetText>) -> Self {
        let text = text.into();
        let id_source = Id::new(text.text());
        Self {
            text,
            default_open: false,
            open: None,
            id_source,
            enabled: true,
            selectable: false,
            selected: false,
            show_background: false,
            icon: None,
            display_event: None,
        }
    }

    /// By default, the [`CollapsingHeader`] is collapsed.
    /// Call `.default_open(true)` to change this.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Calling `.open(Some(true))` will make the collapsing header open this frame (or stay open).
    ///
    /// Calling `.open(Some(false))` will make the collapsing header close this frame (or stay closed).
    ///
    /// Calling `.open(None)` has no effect (default).
    pub fn open(mut self, open: Option<bool>) -> Self {
        self.open = open;
        self
    }

    pub fn display(mut self, new_event: &mut Option<WidgetDisplayEvent>) -> Self {
        self.display_event = new_event.take();
        self
    }

    /// Explicitly set the source of the [`Id`] of this widget, instead of using title label.
    /// This is useful if the title label is dynamic or not unique.
    pub fn id_source(mut self, id_source: impl Hash) -> Self {
        self.id_source = Id::new(id_source);
        self
    }

    /// If you set this to `false`, the [`CollapsingHeader`] will be grayed out and un-clickable.
    ///
    /// This is a convenience for [`Ui::set_enabled`].
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Can the [`CollapsingHeader`] be selected by clicking it? Default: `false`.
    #[deprecated = "Use the more powerful egui::collapsing_header::CollapsingState::show_header"] // Deprecated in 2022-04-28, before egui 0.18
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// If you set this to 'true', the [`CollapsingHeader`] will be shown as selected.
    ///
    /// Example:
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// let mut selected = false;
    /// let response = egui::CollapsingHeader::new("Select and open me")
    ///     .selectable(true)
    ///     .selected(selected)
    ///     .show(ui, |ui| ui.label("Body"));
    /// if response.header_response.clicked() {
    ///     selected = true;
    /// }
    /// # });
    /// ```
    #[deprecated = "Use the more powerful egui::collapsing_header::CollapsingState::show_header"] // Deprecated in 2022-04-28, before egui 0.18
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Should the [`CollapsingHeader`] show a background behind it? Default: `false`.
    ///
    /// To show it behind all [`CollapsingHeader`] you can just use:
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.visuals_mut().collapsing_header_frame = true;
    /// # });
    /// ```
    pub fn show_background(mut self, show_background: bool) -> Self {
        self.show_background = show_background;
        self
    }

    /// Use the provided function to render a different [`CollapsingHeader`] icon.
    /// Defaults to a triangle that animates as the [`CollapsingHeader`] opens and closes.
    ///
    /// For example:
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// fn circle_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    ///     let stroke = ui.style().interact(&response).fg_stroke;
    ///     let radius = egui::lerp(2.0..=3.0, openness);
    ///     ui.painter().circle_filled(response.rect.center(), radius, stroke.color);
    /// }
    ///
    /// egui::CollapsingHeader::new("Circles")
    ///   .icon(circle_icon)
    ///   .show(ui, |ui| { ui.label("Hi!"); });
    /// # });
    /// ```
    pub fn icon(mut self, icon_fn: impl FnOnce(&mut Ui, f32, &Response) + 'static) -> Self {
        self.icon = Some(Box::new(icon_fn));
        self
    }
}

struct Prepared {
    header_response: Response,
    state: WidgetCollapsingState,
    openness: f32,
}

impl CollapsingHeader {
    fn begin(self, ui: &mut Ui) -> Prepared {
        assert!(
            ui.layout().main_dir().is_vertical(),
            "Horizontal collapsing is unimplemented"
        );
        let Self {
            icon,
            text,
            default_open,
            open,
            id_source,
            enabled: _,
            selectable,
            selected,
            show_background,
            display_event,
        } = self;
        // TODO(emilk): horizontal layout, with icon and text as labels. Insert background behind using Frame.

        let id = ui.make_persistent_id(id_source);
        let button_padding = ui.spacing().button_padding;

        let available = ui.available_rect_before_wrap();
        let text_pos = available.min + vec2(ui.spacing().indent, 0.0);
        let wrap_width = available.right() - text_pos.x;
        let wrap = Some(false);
        let text = text.into_galley(ui, wrap, wrap_width, TextStyle::Button);
        let text_max_x = text_pos.x + text.size().x;

        let mut desired_width = text_max_x + button_padding.x - available.left();
        if ui.visuals().collapsing_header_frame {
            desired_width = desired_width.max(available.width()); // fill full width
        }

        let mut desired_size = vec2(desired_width, text.size().y + 2.0 * button_padding.y);
        desired_size = desired_size.at_least(ui.spacing().interact_size);
        let (_, rect) = ui.allocate_space(desired_size);

        let mut header_response = ui.interact(rect, id, Sense::click());
        let text_pos = pos2(
            text_pos.x,
            header_response.rect.center().y - text.size().y / 2.0,
        );

        let mut state = WidgetCollapsingState::load(ui.ctx(), id, default_open);

        let request_repaint = display_event.is_some();

        match display_event {
            Some(WidgetDisplayEvent::Expand) => state.set_open(true),
            Some(WidgetDisplayEvent::Collapse) => state.set_open(false),
            Some(WidgetDisplayEvent::ToggleCollapse) => state.set_open(!state.is_open()),
            None => {}
        }

        if request_repaint {
            ui.ctx().request_repaint();
        } else if let Some(open) = open {
            if open != state.is_open() {
                state.toggle_open(ui);
                header_response.mark_changed();
            }
        } else if header_response.clicked() {
            state.toggle_open(ui);
            header_response.mark_changed();
        }

        header_response
            .widget_info(|| WidgetInfo::labeled(WidgetType::CollapsingHeader, text.text()));

        let openness = state.openness(ui.ctx());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact_selectable(&header_response, selected);

            if ui.visuals().collapsing_header_frame || show_background {
                ui.painter().add(epaint::RectShape::new(
                    header_response.rect.expand(visuals.expansion),
                    visuals.rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                ));
            }

            if selected || selectable && (header_response.hovered() || header_response.has_focus())
            {
                let rect = rect.expand(visuals.expansion);

                ui.painter()
                    .rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);
            }

            {
                let (mut icon_rect, _) = ui.spacing().icon_rectangles(header_response.rect);
                icon_rect.set_center(pos2(
                    header_response.rect.left() + ui.spacing().indent / 2.0,
                    header_response.rect.center().y,
                ));
                let icon_response = header_response.clone().with_new_rect(icon_rect);
                if let Some(icon) = icon {
                    icon(ui, openness, &icon_response);
                } else {
                    CommonCollapse::paint_default_icon(ui, openness, &icon_response);
                }
            }

            text.paint_with_visuals(ui.painter(), text_pos, &visuals);
        }

        Prepared {
            header_response,
            state,
            openness,
        }
    }

    #[inline]
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        self.show_dyn(ui, Box::new(add_body), true)
    }

    #[inline]
    pub fn show_unindented<R>(
        self,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        self.show_dyn(ui, Box::new(add_body), false)
    }

    fn show_dyn<'c, R>(
        self,
        ui: &mut Ui,
        add_body: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
        indented: bool,
    ) -> CollapsingResponse<R> {
        // Make sure body is bellow header,
        // and make sure it is one unit (necessary for putting a [`CollapsingHeader`] in a grid).
        ui.vertical(|ui| {
            ui.set_enabled(self.enabled);

            let Prepared {
                header_response,
                mut state,
                openness,
            } = self.begin(ui); // show the header

            let ret_response = if indented {
                CommonCollapse::show_body_indented(&mut state, &header_response, ui, add_body)
            } else {
                CommonCollapse::show_body_unindented(&mut state, ui, add_body)
            };

            if let Some(ret_response) = ret_response {
                CollapsingResponse {
                    header_response,
                    body_response: Some(ret_response.response),
                    body_returned: Some(ret_response.inner),
                    openness,
                }
            } else {
                CollapsingResponse {
                    header_response,
                    body_response: None,
                    body_returned: None,
                    openness,
                }
            }
        })
        .inner
    }
}

/// The response from showing a [`CollapsingHeader`].
pub struct CollapsingResponse<R> {
    /// Response of the actual clickable header.
    pub header_response: Response,

    /// None iff collapsed.
    pub body_response: Option<Response>,

    /// None iff collapsed.
    pub body_returned: Option<R>,

    /// 0.0 if fully closed, 1.0 if fully open, and something in-between while animating.
    pub openness: f32,
}

impl<R> CollapsingResponse<R> {
    /// Was the [`CollapsingHeader`] fully closed (and not being animated)?
    pub fn fully_closed(&self) -> bool {
        self.openness <= 0.0
    }

    /// Was the [`CollapsingHeader`] fully open (and not being animated)?
    pub fn fully_open(&self) -> bool {
        self.openness >= 1.0
    }
}
