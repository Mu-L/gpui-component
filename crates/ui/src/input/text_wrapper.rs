use std::ops::Range;

use crate::input::LineColumn;
use gpui::{App, Font, LineFragment, Pixels, SharedString};

#[allow(unused)]
pub(super) struct LineWrap {
    /// The number of soft wrapped lines of this line (Not include first line.)
    pub(super) wrap_lines: usize,
    /// The range of the line text in the entire text.
    pub(super) range: Range<usize>,
}

impl LineWrap {
    pub(super) fn height(&self, line_height: Pixels) -> Pixels {
        line_height * (self.wrap_lines + 1)
    }
}

/// Used to prepare the text with soft_wrap to be get lines to displayed in the TextArea
///
/// After use lines to calculate the scroll size of the TextArea
pub(super) struct TextWrapper {
    pub(super) text: SharedString,
    /// The wrapped lines, value is start and end index of the line.
    pub(super) wrapped_lines: Vec<Range<usize>>,
    /// The lines by split \n
    pub(super) lines: Vec<LineWrap>,
    pub(super) font: Font,
    pub(super) font_size: Pixels,
    /// If is none, it means the text is not wrapped
    pub(super) wrap_width: Option<Pixels>,
}

#[allow(unused)]
impl TextWrapper {
    pub(super) fn new(font: Font, font_size: Pixels, wrap_width: Option<Pixels>) -> Self {
        Self {
            text: SharedString::default(),
            font,
            font_size,
            wrap_width,
            wrapped_lines: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub(super) fn set_wrap_width(&mut self, wrap_width: Option<Pixels>, cx: &mut App) {
        self.wrap_width = wrap_width;
        self.update(&self.text.clone(), true, cx);
    }

    pub(super) fn set_font(&mut self, font: Font, font_size: Pixels, cx: &mut App) {
        self.font = font;
        self.font_size = font_size;
        self.update(&self.text.clone(), true, cx);
    }

    /// Update the text wrapper and recalculate the wrapped lines.
    ///
    /// If the `text` is the same as the current text, do nothing.
    pub(super) fn update(&mut self, text: &SharedString, force: bool, cx: &mut App) {
        if &self.text == text && !force {
            return;
        }

        let mut wrapped_lines = vec![];
        let mut lines = vec![];
        let wrap_width = self.wrap_width.unwrap_or(Pixels::MAX);
        let mut line_wrapper = cx
            .text_system()
            .line_wrapper(self.font.clone(), self.font_size);

        let mut prev_line_ix = 0;
        for line in text.split('\n') {
            let mut line_wraps = vec![];
            let mut prev_boundary_ix = 0;

            // Here only have wrapped line, if there is no wrap meet, the `line_wraps` result will empty.
            for boundary in line_wrapper.wrap_line(&[LineFragment::text(line)], wrap_width) {
                line_wraps.push(prev_boundary_ix..boundary.ix);
                prev_boundary_ix = boundary.ix;
            }

            lines.push(LineWrap {
                wrap_lines: line_wraps.len(),
                range: prev_line_ix..prev_line_ix + line.len(),
            });

            wrapped_lines.extend(line_wraps);
            // Reset of the line
            if !line[prev_boundary_ix..].is_empty() || prev_boundary_ix == 0 {
                wrapped_lines.push(prev_line_ix + prev_boundary_ix..prev_line_ix + line.len());
            }

            prev_line_ix += line.len() + 1;
        }

        self.text = text.clone();
        self.wrapped_lines = wrapped_lines;
        self.lines = lines;
    }

    /// Returns the line and column (1-based) of the given offset (Entire text).
    pub(super) fn line_column(&self, offset: usize) -> LineColumn {
        if self.lines.is_empty() {
            return LineColumn::default();
        }

        let line = self
            .lines
            .binary_search_by_key(&offset, |line| line.range.end)
            .unwrap_or_else(|i| i);
        let column = offset.saturating_sub(self.lines[line].range.start);

        (line + 1, column + 1).into()
    }

    /// Returns the offset of the given line and column (1-based).
    ///
    /// - If the `line` is 0, it will return 0.
    /// - If the `line` is greater than the number of lines, it will return
    ///   the length of the text.
    pub(super) fn offset_for_line_column(&self, line: usize, column: usize) -> Option<usize> {
        if line == 0 || self.lines.is_empty() {
            return None;
        }

        let line = line.saturating_sub(1);
        if line >= self.lines.len() {
            return Some(self.text.len());
        }

        let Some(line_wrap) = &self.lines.get(line) else {
            return None;
        };

        let offset = line_wrap.range.start;
        if column == 0 {
            return Some(offset);
        }
        let offset = offset + column.saturating_sub(1).min(line_wrap.range.len());

        Some(offset)
    }
}
