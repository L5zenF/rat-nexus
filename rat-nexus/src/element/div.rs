use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, BorderType};
use crate::element::{Element, IntoElement};

pub struct Div {
    children: Vec<Box<dyn Element>>,
    style: Style,
    direction: Direction,
    width_constraint: Constraint,
    height_constraint: Constraint,
    // Block properties
    borders: Borders,
    border_style: Style,
    border_type: BorderType,
    title: Option<String>,
    padding: ratatui::widgets::Padding,
    margin: u16,
}

pub fn div() -> Div {
    Div::default()
}

impl Default for Div {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            style: Style::default(),
            direction: Direction::Vertical,
            width_constraint: Constraint::Min(0),
            height_constraint: Constraint::Min(0),
            borders: Borders::NONE,
            border_style: Style::default(),
            border_type: BorderType::Plain,
            title: None,
            padding: ratatui::widgets::Padding::ZERO,
            margin: 0,
        }
    }
}

impl Div {
    // --- Layout ---

    pub fn flex(self) -> Self {
        self
    }

    pub fn flex_col(mut self) -> Self {
        self.direction = Direction::Vertical;
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.direction = Direction::Horizontal;
        self
    }

    // --- Sizing ---

    pub fn w_full(mut self) -> Self {
        self.width_constraint = Constraint::Percentage(100);
        self
    }

    pub fn h_full(mut self) -> Self {
        self.height_constraint = Constraint::Percentage(100);
        self
    }

    pub fn w_1_2(mut self) -> Self {
        self.width_constraint = Constraint::Percentage(50);
        self
    }
    
    pub fn h_1_2(mut self) -> Self {
        self.height_constraint = Constraint::Percentage(50);
        self
    }

    pub fn w(mut self, length: u16) -> Self {
        self.width_constraint = Constraint::Length(length);
        self
    }

    pub fn h(mut self, length: u16) -> Self {
        self.height_constraint = Constraint::Length(length);
        self
    }

    pub fn w_percent(mut self, p: u16) -> Self {
        self.width_constraint = Constraint::Percentage(p);
        self
    }

    pub fn h_percent(mut self, p: u16) -> Self {
        self.height_constraint = Constraint::Percentage(p);
        self
    }

    // --- Styling ---

    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.style = self.style.add_modifier(Modifier::BOLD);
        self
    }

    pub fn border(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn border_all(mut self) -> Self {
        self.borders = Borders::ALL;
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn p(mut self, constr: u16) -> Self {
        self.padding = ratatui::widgets::Padding::new(constr, constr, constr, constr);
        self
    }

    pub fn px(mut self, constr: u16) -> Self {
        self.padding.left = constr;
        self.padding.right = constr;
        self
    }

    pub fn py(mut self, constr: u16) -> Self {
        self.padding.top = constr;
        self.padding.bottom = constr;
        self
    }

    pub fn m(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    // --- Children ---

    pub fn child(mut self, child: impl IntoElement + 'static) -> Self {
        self.children.push(Box::new(child.into_element()));
        self
    }

    pub fn children(mut self, children: Vec<impl IntoElement + 'static>) -> Self {
        for child in children {
            self.children.push(Box::new(child.into_element()));
        }
        self
    }
}

impl Element for Div {
    fn width(&self) -> Constraint {
        self.width_constraint
    }

    fn height(&self) -> Constraint {
        self.height_constraint
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // 1. Render Block (background, borders)
        let block = Block::default()
            .style(self.style)
            .borders(self.borders)
            .border_style(self.border_style)
            .border_type(self.border_type)
            .padding(self.padding);
        
        let block = if let Some(title) = &self.title {
            block.title(title.clone())
        } else {
            block
        };

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // 2. Compute Layout for Children
        if self.children.is_empty() {
            return;
        }

        // Apply margin to inner area layout calculation? 
        // No, margin is usually OUTSIDE the element or inside layout container.
        // Ratatui Layout has margin() which shrinks the area BEFORE splitting.
        // So we should apply margin to the layout.
        
        let constraints: Vec<Constraint> = self.children.iter().map(|c| {
            if self.direction == Direction::Vertical {
                c.height()
            } else {
                c.width()
            }
        }).collect();

        let layout = Layout::default()
            .direction(self.direction)
            .margin(self.margin)
            .constraints(constraints);

        let chunks = layout.split(inner_area);

        // 3. Render Children
        for (i, child) in self.children.iter_mut().enumerate() {
            if i < chunks.len() {
                child.render(frame, chunks[i]);
            }
        }
    }
}
