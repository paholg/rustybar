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

/// Position used to park children that should not be displayed. They are
/// skipped in `draw` because they never intersect the viewport.
const HIDDEN: Point = Point::new(-1e6, -1e6);

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
        let spacing = self.spacing;

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

            let new_pl: Vec<usize> = self.pinned.iter().copied().filter(|&i| i < lo).collect();
            let new_pr: Vec<usize> = self.pinned.iter().copied().filter(|&i| i > hi).collect();
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

        let nodes = nodes
            .into_iter()
            .zip(positions)
            .map(|(node, position)| node.move_to(position.unwrap_or(HIDDEN)))
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
        for ((child, tree), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
        {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
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
