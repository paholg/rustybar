use iced::{
    Element, Event, Length, Point, Rectangle, Size,
    advanced::{
        Clipboard, Shell, Widget,
        layout::{Layout, Limits, Node},
        mouse, renderer,
        widget::{Operation, Tree},
    },
};

/// A row that limits itself to `max_fraction` of the available width by
/// showing only the children around `focus` that fit. Hidden ranges are
/// indicated by separator elements ("…"), and `pinned` children are always
/// shown, relocated to the near edge when they fall outside the visible
/// window.
pub struct OverflowRow<'a, Message, Theme, Renderer> {
    /// The row children, followed by the left and right separator elements.
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    focus: Option<usize>,
    pinned: Vec<usize>,
    max_fraction: f32,
    spacing: f32,
}

impl<'a, Message, Theme, Renderer> OverflowRow<'a, Message, Theme, Renderer> {
    pub fn new(
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
        separators: [Element<'a, Message, Theme, Renderer>; 2],
        focus: Option<usize>,
        mut pinned: Vec<usize>,
        max_fraction: f32,
        spacing: f32,
    ) -> Self {
        let mut children: Vec<_> = children.into_iter().collect();
        children.extend(separators);
        pinned.sort_unstable();

        Self {
            children,
            focus,
            pinned,
            max_fraction,
            spacing,
        }
    }

    /// Picks which children to display, in display order, when the full row
    /// does not fit in `cap`. Separators are referenced by their indices
    /// (`n` for the left one, `n + 1` for the right one).
    ///
    /// Builds a contiguous window around the focus, reserving space for
    /// pinned children outside it and for separators marking hidden ranges.
    /// Reservations shrink the window, which can push more pinned children
    /// outside, so iterate to a fixpoint; reservations only grow, so this
    /// terminates.
    fn select(&self, cap: f32, widths: &[f32]) -> Vec<usize> {
        let n = self.children.len() - 2;
        let focus = self.focus.unwrap_or(0).min(n - 1);
        select(n, focus, &self.pinned, widths, self.spacing, cap)
    }
}

/// Picks which children to display, in display order, when the full row does
/// not fit in `cap`. `widths` holds the `n` child widths followed by the two
/// separator widths; separators are referenced by index `n` (left) and `n + 1`
/// (right). See [`OverflowRow::select`] for the high-level description.
fn select(
    n: usize,
    focus: usize,
    pinned: &[usize],
    widths: &[f32],
    spacing: f32,
    cap: f32,
) -> Vec<usize> {
    let mut pinned_left: Vec<usize> = Vec::new();
    let mut pinned_right: Vec<usize> = Vec::new();
    let mut ellipsis_left = false;
    let mut ellipsis_right = false;
    let (mut lo, mut hi) = (focus, focus);

    for _ in 0..n + 2 {
        let mut reserve = 0.0;
        for &i in pinned_left.iter().chain(&pinned_right) {
            reserve += widths[i] + spacing;
        }
        if ellipsis_left {
            reserve += widths[n] + spacing;
        }
        if ellipsis_right {
            reserve += widths[n + 1] + spacing;
        }

        // Grow the window around the focus, keeping it roughly centered
        // by expanding whichever side is currently narrower.
        let avail = cap - reserve;
        lo = focus;
        hi = focus;
        let mut used = widths[focus];
        let (mut used_left, mut used_right) = (0.0, 0.0);
        loop {
            let can_left = lo > 0 && used + spacing + widths[lo - 1] <= avail;
            let can_right = hi + 1 < n && used + spacing + widths[hi + 1] <= avail;
            let go_left = match (can_left, can_right) {
                (false, false) => break,
                (true, false) => true,
                (false, true) => false,
                (true, true) => used_left <= used_right,
            };
            if go_left {
                lo -= 1;
                used += spacing + widths[lo];
                used_left += widths[lo];
            } else {
                hi += 1;
                used += spacing + widths[hi];
                used_right += widths[hi];
            }
        }

        let new_pl: Vec<usize> = pinned.iter().copied().filter(|&i| i < lo).collect();
        let new_pr: Vec<usize> = pinned.iter().copied().filter(|&i| i > hi).collect();
        let new_el = (0..lo).any(|i| !new_pl.contains(&i));
        let new_er = (hi + 1..n).any(|i| !new_pr.contains(&i));

        if new_pl == pinned_left
            && new_pr == pinned_right
            && new_el == ellipsis_left
            && new_er == ellipsis_right
        {
            break;
        }
        pinned_left = new_pl;
        pinned_right = new_pr;
        ellipsis_left = new_el;
        ellipsis_right = new_er;
    }

    let mut order = pinned_left;
    if ellipsis_left {
        order.push(n);
    }
    order.extend(lo..=hi);
    if ellipsis_right {
        order.push(n + 1);
    }
    order.extend(pinned_right);
    order
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for OverflowRow<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let n = self.children.len() - 2;
        let unbounded = Limits::new(Size::ZERO, Size::new(f32::INFINITY, limits.max().height));
        let nodes: Vec<Node> = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(child, tree)| child.as_widget_mut().layout(tree, renderer, &unbounded))
            .collect();
        let widths: Vec<f32> = nodes.iter().map(|node| node.size().width).collect();

        let cap = self.max_fraction * limits.max().width;
        let total =
            widths[..n].iter().sum::<f32>() + self.spacing * n.saturating_sub(1) as f32;

        let visible: Vec<usize> = if n == 0 {
            Vec::new()
        } else if total <= cap || !cap.is_finite() {
            (0..n).collect()
        } else {
            self.select(cap, &widths)
        };

        let mut x = 0.0;
        let mut height = 0.0f32;
        let mut positions: Vec<Option<Point>> = vec![None; self.children.len()];
        for &i in &visible {
            positions[i] = Some(Point::new(x, 0.0));
            x += widths[i] + self.spacing;
            height = height.max(nodes[i].size().height);
        }
        let width = (x - self.spacing).max(0.0);

        // Off-window children are parked just past the right edge and clipped
        // away in `draw`. They are still laid out and drawn every frame so the
        // set of emitted primitives stays stable in count and order: iced's
        // damage tracking pairs primitives positionally, so skipping children
        // (or teleporting them far off-screen) mispairs them and leaves stale
        // glyphs on screen.
        let hidden = Point::new(width, 0.0);
        let nodes = nodes
            .into_iter()
            .zip(positions)
            .map(|(node, position)| node.move_to(position.unwrap_or(hidden)))
            .collect();

        Node::with_children(Size::new(width, height), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };

        // Draw every child, clipped to our bounds. Off-window children are
        // parked past the right edge and clipped out, but still emitted so the
        // primitive list stays stable for iced's positional damage diff.
        renderer.with_layer(clip, |renderer| {
            for ((child, tree), child_layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, child_layout, cursor, &clip);
            }
        });
    }
}

impl<'a, Message, Theme, Renderer> From<OverflowRow<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(row: OverflowRow<'a, Message, Theme, Renderer>) -> Self {
        Element::new(row)
    }
}

#[cfg(test)]
mod tests {
    use super::{select, OverflowRow};
    use iced::advanced::widget::Tree;
    use iced::widget::{container, text, Stack};
    use iced::{Element, Length, Size, Theme};

    const SPACING: f32 = 12.0;

    /// A child of fixed width `w`; with the null renderer this measures to `w`
    /// regardless of text, so overflow actually triggers under test.
    fn box_of(w: f32) -> Element<'static, (), Theme, ()> {
        container(text("")).width(Length::Fixed(w)).height(Length::Fixed(10.0)).into()
    }

    /// Mirrors the real workspace child: a fixed-width label with a
    /// `Length::Fill` underline overlaid in a `Stack`. Exercises the path where
    /// a Fill child is measured under the unbounded (infinite-width) limits.
    fn stack_box(w: f32) -> Element<'static, (), Theme, ()> {
        Stack::new()
            .push(container(text("")).width(Length::Fixed(w)).height(Length::Fixed(10.0)))
            .push(
                container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .into()
    }

    /// Lays out a real `OverflowRow` headlessly and returns, for each child
    /// index, its on-screen x (or `None` if parked off-screen). Index `n` and
    /// `n + 1` are the separators.
    fn layout_positions(
        make: fn(f32) -> Element<'static, (), Theme, ()>,
        child_widths: &[f32],
        sep_widths: [f32; 2],
        focus: Option<usize>,
        pinned: Vec<usize>,
        max_fraction: f32,
        avail_width: f32,
    ) -> Vec<Option<f32>> {
        use iced::advanced::layout::Limits;

        let children: Vec<Element<'static, (), Theme, ()>> =
            child_widths.iter().map(|&w| make(w)).collect();
        let separators = [box_of(sep_widths[0]), box_of(sep_widths[1])];
        let row = OverflowRow::new(children, separators, focus, pinned, max_fraction, SPACING);

        let mut element: Element<'static, (), Theme, ()> = row.into();
        let mut tree = Tree::new(&element);
        let limits = Limits::new(Size::ZERO, Size::new(avail_width, 100.0));
        let node = element
            .as_widget_mut()
            .layout(&mut tree, &(), &limits);

        let w = node.size().width;
        node.children()
            .iter()
            .map(|c| {
                let b = c.bounds();
                // off-window children are parked at the right edge (x == width)
                if b.x >= w { None } else { Some(b.x) }
            })
            .collect()
    }

    /// Lay out the real widget under overflow and assert it places each chosen
    /// child exactly once, at a distinct x — i.e. nothing is drawn twice.
    fn assert_each_child_once(make: fn(f32) -> Element<'static, (), Theme, ()>) {
        let child_widths = [52.0, 52.0, 64.0]; // devc, main, pread
        let sep = [12.0, 12.0];
        let n = child_widths.len();

        for focus in 0..n {
            for avail in [80.0, 120.0, 160.0, 200.0, 260.0, 400.0f32] {
                let max_fraction = 1.0;
                let cap = max_fraction * avail;
                let positions = layout_positions(
                    make,
                    &child_widths,
                    sep,
                    Some(focus),
                    vec![],
                    max_fraction,
                    avail,
                );

                let mut widths = child_widths.to_vec();
                widths.extend(sep);
                let total: f32 = child_widths.iter().sum::<f32>() + SPACING * (n - 1) as f32;
                let expected: Vec<usize> = if total <= cap {
                    (0..n).collect()
                } else {
                    select(n, focus, &[], &widths, SPACING, cap)
                };

                let on_screen: Vec<usize> = positions
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| p.map(|_| i))
                    .collect();

                // the widget shows exactly what the selection chose
                let mut want = expected.clone();
                want.sort_unstable();
                let mut got = on_screen.clone();
                got.sort_unstable();
                assert_eq!(got, want, "focus={focus} avail={avail}: shown set mismatch");

                // every on-screen child sits at a unique x (no overlap / double-draw)
                let mut xs: Vec<f32> =
                    on_screen.iter().map(|&i| positions[i].unwrap()).collect();
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                for pair in xs.windows(2) {
                    assert!(pair[1] > pair[0], "two children share an x: {xs:?}");
                }
            }
        }
    }

    #[test]
    fn layout_draws_each_child_once() {
        assert_each_child_once(box_of);
    }

    /// Same, but with the real workspace child shape (a `Stack` carrying a
    /// `Length::Fill` underline) — confirms the Fill overlay neither inflates
    /// the measured width nor causes a double-draw under overflow.
    #[test]
    fn layout_draws_each_stack_child_once() {
        assert_each_child_once(stack_box);
    }

    /// Splits a `select` result into (real child indices, separator indices),
    /// given `n` real children. Separators are `n` (left) and `n + 1` (right).
    fn split(order: &[usize], n: usize) -> (Vec<usize>, Vec<usize>) {
        order.iter().partition(|&&i| i < n)
    }

    /// Builds a `widths` slice: `n` real widths plus two separator widths.
    fn widths(reals: &[f32], sep: f32) -> Vec<f32> {
        let mut w = reals.to_vec();
        w.push(sep);
        w.push(sep);
        w
    }

    /// Asserts the universal invariants `select` must uphold, returning the
    /// real (workspace) indices in display order for further checks.
    fn check_invariants(
        n: usize,
        focus: usize,
        pinned: &[usize],
        order: &[usize],
    ) -> Vec<usize> {
        let ctx = format!("n={n} focus={focus} pinned={pinned:?} order={order:?}");

        // Every index is a valid child or separator slot.
        for &i in order {
            assert!(i < n + 2, "out-of-range index {i} ({ctx})");
        }

        let (reals, seps) = split(order, n);

        // No child is shown twice. THIS is the "main shows up twice" bug.
        let mut sorted = reals.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), reals.len(), "duplicate child index ({ctx})");

        // No separator is shown twice, and there are at most two.
        let mut sseps = seps.clone();
        sseps.sort_unstable();
        sseps.dedup();
        assert_eq!(sseps.len(), seps.len(), "duplicate separator ({ctx})");
        assert!(seps.len() <= 2, "too many separators ({ctx})");

        // The focused child is always visible.
        assert!(reals.contains(&focus), "focus not shown ({ctx})");

        // Every pinned child is always visible.
        for &p in pinned {
            if p < n {
                assert!(reals.contains(&p), "pinned {p} not shown ({ctx})");
            }
        }

        reals
    }

    /// Exhaustively exercises every small configuration: each `n`, each focus,
    /// every subset of pins, a few width profiles, and a sweep of caps.
    #[test]
    fn invariants_hold_exhaustively() {
        let sep = 8.0;
        let width_profiles: &[fn(usize) -> Vec<f32>] = &[
            |n| vec![20.0; n],                                   // uniform
            |n| (0..n).map(|i| 10.0 + 10.0 * i as f32).collect(), // increasing
            |n| (0..n).map(|i| 10.0 + 10.0 * (n - i) as f32).collect(), // decreasing
            |n| (0..n).map(|i| if i % 2 == 0 { 12.0 } else { 40.0 }).collect(), // alternating
        ];

        for n in 1..=8usize {
            for focus in 0..n {
                // every subset of {0..n} as the pinned set
                for mask in 0u32..(1 << n) {
                    let pinned: Vec<usize> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
                    for profile in width_profiles {
                        let reals = profile(n);
                        let w = widths(&reals, sep);
                        // sweep caps from too-small to comfortably-large
                        let total: f32 = reals.iter().sum::<f32>() + SPACING * (n - 1) as f32;
                        let mut cap = 0.0;
                        while cap <= total + 40.0 {
                            let order = select(n, focus, &pinned, &w, SPACING, cap);
                            check_invariants(n, focus, &pinned, &order);
                            cap += 1.0;
                        }
                    }
                }
            }
        }
    }

    /// With no pins, the shown workspaces must be exactly a contiguous block
    /// around the focus — "only the closest to the current one".
    #[test]
    fn unpinned_window_is_contiguous_around_focus() {
        let sep = 8.0;
        for n in 1..=8usize {
            for focus in 0..n {
                let reals: Vec<f32> = (0..n).map(|i| 12.0 + 7.0 * (i % 3) as f32).collect();
                let w = widths(&reals, sep);
                let total: f32 = reals.iter().sum::<f32>() + SPACING * (n - 1) as f32;
                let mut cap = 0.0;
                while cap <= total + 40.0 {
                    let order = select(n, focus, &[], &w, SPACING, cap);
                    let reals_shown = check_invariants(n, focus, &[], &order);

                    // contiguous: shown reals form a run of consecutive ints
                    let lo = *reals_shown.iter().min().unwrap();
                    let hi = *reals_shown.iter().max().unwrap();
                    let expected: Vec<usize> = (lo..=hi).collect();
                    assert_eq!(
                        reals_shown, expected,
                        "non-contiguous unpinned window: n={n} focus={focus} cap={cap} order={order:?}"
                    );
                    assert!(lo <= focus && focus <= hi);

                    // a leading separator implies something is actually hidden
                    // on the left (and likewise on the right)
                    if order.first() == Some(&n) {
                        assert!(lo > 0, "left ellipsis but nothing hidden left: {order:?}");
                    }
                    if order.last() == Some(&(n + 1)) {
                        assert!(hi + 1 < n, "right ellipsis but nothing hidden right: {order:?}");
                    }
                    cap += 1.0;
                }
            }
        }
    }

    /// Reproduction of the reported video: workspaces `devc, main, pread`
    /// (unique indices, none pinned), focus on the right, under overflow.
    /// The bar must never render the same workspace twice.
    #[test]
    fn video_repro_no_duplicate_under_overflow() {
        let names = ["devc", "main", "pread"];
        let n = names.len();
        let reals = vec![52.0, 52.0, 64.0];
        let w = widths(&reals, 12.0);

        for focus in 0..n {
            let mut cap = 0.0;
            while cap <= 300.0 {
                let order = select(n, focus, &[], &w, SPACING, cap);
                let labels: Vec<&str> = order
                    .iter()
                    .map(|&i| if i < n { names[i] } else { "…" })
                    .collect();
                let shown: Vec<&str> = labels.iter().copied().filter(|&l| l != "…").collect();
                let mut uniq = shown.clone();
                uniq.sort_unstable();
                uniq.dedup();
                assert_eq!(
                    uniq.len(),
                    shown.len(),
                    "workspace shown twice: focus={focus} cap={cap} -> {labels:?}"
                );
                cap += 1.0;
            }
        }
    }

    // ---- Full update -> layout -> draw pipeline, across frames ----------
    //
    // The tests above cover selection, data, and layout positions. This block
    // drives the actual draw path over consecutive frames through one persisted
    // `Tree`, recording what is painted, to catch any stale-state / tree-diff
    // reuse bug (e.g. a workspace rendering a previous frame's identity).

    use iced::advanced::renderer::{self, Quad};
    use iced::advanced::text as adv_text;
    use iced::advanced::{layout::Limits, mouse, Layout};
    use iced::{Background, Color, Font, Pixels, Point, Rectangle};
    use std::cell::RefCell;

    /// A renderer that records the background color and bounds of every quad it
    /// is asked to fill. A workspace child paints exactly one such quad.
    #[derive(Default)]
    struct Rec {
        quads: RefCell<Vec<(Color, Rectangle)>>,
    }

    impl iced::advanced::Renderer for Rec {
        fn start_layer(&mut self, _bounds: Rectangle) {}
        fn end_layer(&mut self) {}
        fn start_transformation(&mut self, _t: iced::Transformation) {}
        fn end_transformation(&mut self) {}
        fn reset(&mut self, _b: Rectangle) {}
        fn fill_quad(&mut self, quad: Quad, background: impl Into<Background>) {
            if let Background::Color(c) = background.into() {
                self.quads.borrow_mut().push((c, quad.bounds));
            }
        }
        fn allocate_image(
            &mut self,
            handle: &iced::advanced::image::Handle,
            callback: impl FnOnce(
                Result<iced::advanced::image::Allocation, iced::advanced::image::Error>,
            ) + Send
            + 'static,
        ) {
            #[allow(unsafe_code)]
            callback(Ok(unsafe {
                iced::advanced::image::allocate(handle, Size::new(1, 1))
            }));
        }
    }

    impl adv_text::Renderer for Rec {
        type Font = Font;
        type Paragraph = ();
        type Editor = ();
        const ICON_FONT: Font = Font::DEFAULT;
        const CHECKMARK_ICON: char = '0';
        const ARROW_DOWN_ICON: char = '0';
        const SCROLL_UP_ICON: char = '0';
        const SCROLL_DOWN_ICON: char = '0';
        const SCROLL_LEFT_ICON: char = '0';
        const SCROLL_RIGHT_ICON: char = '0';
        const ICED_LOGO: char = '0';
        fn default_font(&self) -> Font {
            Font::default()
        }
        fn default_size(&self) -> Pixels {
            Pixels(16.0)
        }
        fn fill_paragraph(&mut self, _p: &(), _pos: Point, _c: Color, _clip: Rectangle) {}
        fn fill_editor(&mut self, _e: &(), _pos: Point, _c: Color, _clip: Rectangle) {}
        fn fill_text(&mut self, _t: adv_text::Text, _pos: Point, _c: Color, _clip: Rectangle) {}
    }

    /// Encodes a workspace identity into a unique recoverable color.
    fn color_for(id: usize) -> Color {
        Color::from_rgb((id as f32 + 1.0) / 200.0, 0.0, 0.0)
    }

    fn id_of(color: Color) -> usize {
        (color.r * 200.0).round() as usize - 1
    }

    /// A workspace child carrying `id` as its background color (recoverable in
    /// the recording renderer) and fixed width `w`.
    fn colored(id: usize, w: f32) -> Element<'static, (), Theme, Rec> {
        let c = color_for(id);
        container(text(""))
            .width(Length::Fixed(w))
            .height(Length::Fixed(10.0))
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(c)),
                ..container::Style::default()
            })
            .into()
    }

    /// Builds one frame's widget: workspaces in `order` (by id) with `widths`,
    /// plus two separators (ids 100, 101).
    fn frame(
        order: &[(usize, f32)],
        sep_w: f32,
        focus: Option<usize>,
        pinned: Vec<usize>,
        max_fraction: f32,
    ) -> Element<'static, (), Theme, Rec> {
        let children: Vec<Element<'static, (), Theme, Rec>> =
            order.iter().map(|&(id, w)| colored(id, w)).collect();
        let separators = [colored(100, sep_w), colored(101, sep_w)];
        OverflowRow::new(children, separators, focus, pinned, max_fraction, SPACING).into()
    }

    /// Lays out and draws `element` against a fresh recorder, returning the ids
    /// painted, in left-to-right order.
    fn render_ids(
        element: &mut Element<'static, (), Theme, Rec>,
        tree: &mut Tree,
        avail: f32,
    ) -> Vec<usize> {
        let renderer = Rec::default();
        let limits = Limits::new(Size::ZERO, Size::new(avail, 100.0));
        let node = {
            let mut r = Rec::default();
            element.as_widget_mut().layout(tree, &mut r, &limits)
        };
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(avail, 100.0));
        let style = renderer::Style {
            text_color: Color::WHITE,
        };
        let width = node.size().width;
        let mut renderer = renderer;
        element.as_widget().draw(
            tree,
            &mut renderer,
            &Theme::Dark,
            &style,
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
        // The recording renderer does not clip, so drop children parked at/past
        // the right edge (x >= width) the way the real clip would.
        let mut painted: Vec<(usize, f32)> = renderer
            .quads
            .borrow()
            .iter()
            .map(|&(c, b)| (id_of(c), b.x))
            .filter(|&(_, x)| x < width)
            .collect();
        painted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        painted.into_iter().map(|(id, _)| id).collect()
    }

    /// Drive several frames through ONE tree, mutating the workspace set the way
    /// niri events would, and assert every frame paints each id at most once and
    /// never a stale (no-longer-present) id. This is the render-layer analogue
    /// of the reported "main shows up twice" bug.
    #[test]
    fn cross_frame_draw_never_duplicates_or_goes_stale() {
        let avail = 150.0; // forces overflow for 3 workspaces
        let sep_w = 12.0;

        // Each frame: (workspaces as (id,width), focus). Ids are stable
        // identities; we shuffle order, change focus, and add/remove to stress
        // the tree diff across frames — exactly what overflow + niri events do.
        let frames: Vec<(Vec<(usize, f32)>, Option<usize>)> = vec![
            (vec![(0, 52.0), (1, 52.0), (2, 64.0)], Some(2)),
            (vec![(0, 52.0), (1, 52.0), (2, 64.0)], Some(1)),
            (vec![(0, 52.0), (1, 52.0), (2, 64.0)], Some(0)),
            (vec![(1, 52.0), (2, 64.0)], Some(1)), // id 0 removed
            (vec![(0, 52.0), (1, 52.0), (2, 64.0), (3, 40.0)], Some(3)), // re-add + new
            (vec![(0, 52.0), (1, 52.0), (2, 64.0)], Some(2)),
        ];

        let mut elem = frame(&frames[0].0, sep_w, frames[0].1, vec![], 1.0);
        let mut tree = Tree::new(&elem);

        for (i, (workspaces, focus)) in frames.iter().enumerate() {
            elem = frame(workspaces, sep_w, *focus, vec![], 1.0);
            elem.as_widget().diff(&mut tree);
            let painted = render_ids(&mut elem, &mut tree, avail);

            let present: Vec<usize> = workspaces.iter().map(|&(id, _)| id).collect();

            // no id painted twice
            let mut uniq = painted.clone();
            uniq.sort_unstable();
            uniq.dedup();
            assert_eq!(
                uniq.len(),
                painted.len(),
                "frame {i}: an id was painted twice: {painted:?}"
            );

            // every painted workspace id (separators are 100/101) is actually
            // present this frame — never a stale identity
            for &id in &painted {
                if id < 100 {
                    assert!(
                        present.contains(&id),
                        "frame {i}: painted stale id {id}, present={present:?}, painted={painted:?}"
                    );
                }
            }

            // the focused workspace is always painted
            if let Some(f) = focus {
                assert!(
                    painted.contains(&present[*f]),
                    "frame {i}: focus id {} not painted: {painted:?}",
                    present[*f]
                );
            }
        }
    }

    // ---- Real-text reproduction: capture the actual drawn label strings ----

    const CHAR_W: f32 = 9.0;

    /// A faithful-enough `text::Paragraph` that stores its content and reports a
    /// width proportional to its length, so overflow triggers on real strings
    /// and the drawn content is recoverable.
    #[derive(Default, Clone)]
    struct ContentPara {
        content: String,
    }

    impl adv_text::Paragraph for ContentPara {
        type Font = Font;
        fn with_text(text: adv_text::Text<&str, Font>) -> Self {
            ContentPara { content: text.content.to_string() }
        }
        fn with_spans<L>(
            _: adv_text::Text<&[adv_text::Span<'_, L, Font>], Font>,
        ) -> Self {
            ContentPara::default()
        }
        fn resize(&mut self, _: Size) {}
        fn compare(&self, _: adv_text::Text<()>) -> adv_text::Difference {
            adv_text::Difference::None
        }
        fn size(&self) -> Pixels {
            Pixels(16.0)
        }
        fn font(&self) -> Font {
            Font::DEFAULT
        }
        fn line_height(&self) -> adv_text::LineHeight {
            adv_text::LineHeight::default()
        }
        fn align_x(&self) -> adv_text::Alignment {
            adv_text::Alignment::Default
        }
        fn align_y(&self) -> iced::alignment::Vertical {
            iced::alignment::Vertical::Top
        }
        fn wrapping(&self) -> adv_text::Wrapping {
            adv_text::Wrapping::default()
        }
        fn shaping(&self) -> adv_text::Shaping {
            adv_text::Shaping::default()
        }
        fn grapheme_position(&self, _: usize, _: usize) -> Option<Point> {
            None
        }
        fn bounds(&self) -> Size {
            Size::new(self.content.chars().count() as f32 * CHAR_W, 10.0)
        }
        fn min_bounds(&self) -> Size {
            self.bounds()
        }
        fn hit_test(&self, _: Point) -> Option<adv_text::Hit> {
            None
        }
        fn hit_span(&self, _: Point) -> Option<usize> {
            None
        }
        fn span_bounds(&self, _: usize) -> Vec<Rectangle> {
            Vec::new()
        }
    }

    #[derive(Default)]
    struct RecText {
        drawn: RefCell<Vec<(String, f32)>>,
    }

    impl iced::advanced::Renderer for RecText {
        fn start_layer(&mut self, _: Rectangle) {}
        fn end_layer(&mut self) {}
        fn start_transformation(&mut self, _: iced::Transformation) {}
        fn end_transformation(&mut self) {}
        fn reset(&mut self, _: Rectangle) {}
        fn fill_quad(&mut self, _: Quad, _: impl Into<Background>) {}
        fn allocate_image(
            &mut self,
            handle: &iced::advanced::image::Handle,
            callback: impl FnOnce(
                Result<iced::advanced::image::Allocation, iced::advanced::image::Error>,
            ) + Send
            + 'static,
        ) {
            #[allow(unsafe_code)]
            callback(Ok(unsafe {
                iced::advanced::image::allocate(handle, Size::new(1, 1))
            }));
        }
    }

    impl adv_text::Renderer for RecText {
        type Font = Font;
        type Paragraph = ContentPara;
        type Editor = ();
        const ICON_FONT: Font = Font::DEFAULT;
        const CHECKMARK_ICON: char = '0';
        const ARROW_DOWN_ICON: char = '0';
        const SCROLL_UP_ICON: char = '0';
        const SCROLL_DOWN_ICON: char = '0';
        const SCROLL_LEFT_ICON: char = '0';
        const SCROLL_RIGHT_ICON: char = '0';
        const ICED_LOGO: char = '0';
        fn default_font(&self) -> Font {
            Font::default()
        }
        fn default_size(&self) -> Pixels {
            Pixels(16.0)
        }
        fn fill_paragraph(
            &mut self,
            p: &ContentPara,
            pos: Point,
            _: Color,
            _: Rectangle,
        ) {
            self.drawn.borrow_mut().push((p.content.clone(), pos.x));
        }
        fn fill_editor(&mut self, _: &(), _: Point, _: Color, _: Rectangle) {}
        fn fill_text(&mut self, _: adv_text::Text, _: Point, _: Color, _: Rectangle) {}
    }

    /// Builds the bar for `labels` (focus index `focus`) and returns the label
    /// strings actually painted, left to right.
    fn drawn_labels(labels: &[&str], focus: usize, avail: f32) -> Vec<String> {
        let children: Vec<Element<'static, (), Theme, RecText>> = labels
            .iter()
            .map(|&l| text(l.to_string()).into())
            .collect();
        let separators = [text("…").into(), text("…").into()];
        let mut element: Element<'static, (), Theme, RecText> =
            OverflowRow::new(children, separators, Some(focus), vec![], 1.0, SPACING).into();

        let mut tree = Tree::new(&element);
        let limits = Limits::new(Size::ZERO, Size::new(avail, 100.0));
        let node = {
            let mut r = RecText::default();
            element.as_widget_mut().layout(&mut tree, &mut r, &limits)
        };
        let layout = Layout::new(&node);
        let width = node.size().width;
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(avail, 100.0));
        let style = renderer::Style { text_color: Color::WHITE };
        let mut renderer = RecText::default();
        element.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &style,
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
        // The recording renderer does not clip, so drop children parked at/past
        // the right edge (x >= width) the way the real clip would.
        let mut painted: Vec<(String, f32)> = renderer
            .drawn
            .borrow()
            .iter()
            .filter(|(_, x)| *x < width)
            .cloned()
            .collect();
        painted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        painted.into_iter().map(|(s, _)| s).collect()
    }

    /// What `select` says should be shown, as label strings, for cross-check.
    /// The reported bug: with focus to the right under overflow, the bar paints
    /// `sftp-table`/`sftp-write` (same length, shared prefix) confused. Drive
    /// the real widget and assert the painted labels match the selection, with
    /// no label painted twice.
    #[test]
    fn real_text_labels_are_not_confused_under_overflow() {
        let labels = ["a", "b", "sftp-table", "sftp-write", "main", "pread", "edit"];
        let n = labels.len();
        let mut failures = Vec::new();

        for focus in 0..n {
            for avail in [90.0, 140.0, 200.0, 280.0, 360.0, 440.0, 520.0, 700.0f32] {
                let painted = drawn_labels(&labels, focus, avail);
                // workspace labels actually painted (drop the … separators)
                let reals: Vec<&str> =
                    painted.iter().map(String::as_str).filter(|s| *s != "…").collect();

                // (1) no workspace label painted twice
                let mut uniq = reals.clone();
                uniq.sort_unstable();
                uniq.dedup();
                if uniq.len() != reals.len() {
                    failures.push(format!(
                        "focus={focus}({}) avail={avail}: label painted twice -> {painted:?}",
                        labels[focus]
                    ));
                    continue;
                }

                // (2) painted labels are a contiguous slice of the data, in
                // order (this is what "only the closest, no confusion" means)
                let idxs: Vec<usize> = reals
                    .iter()
                    .map(|s| labels.iter().position(|l| l == s).unwrap())
                    .collect();
                let contiguous = idxs.windows(2).all(|w| w[1] == w[0] + 1);
                if !contiguous || !idxs.contains(&focus) {
                    failures.push(format!(
                        "focus={focus}({}) avail={avail}: painted={painted:?} idxs={idxs:?}",
                        labels[focus]
                    ));
                }
            }
        }

        assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
    }

    /// `select` must be a pure function: identical inputs yield identical
    /// output every call (rules out internal nondeterminism).
    #[test]
    fn select_is_deterministic() {
        let reals = vec![13.0, 41.0, 13.0, 41.0, 27.0];
        let w = widths(&reals, 8.0);
        for focus in 0..5 {
            for pinned in [vec![], vec![0], vec![0, 4], vec![2], vec![0, 2, 4]] {
                for cap in [10.0, 33.0, 51.0, 77.0, 95.0, 130.0, 180.0f32] {
                    let a = select(5, focus, &pinned, &w, SPACING, cap);
                    let b = select(5, focus, &pinned, &w, SPACING, cap);
                    assert_eq!(a, b, "nondeterministic: focus={focus} pinned={pinned:?} cap={cap}");
                }
            }
        }
    }
}
