#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionRange {
    pub anchor: usize,
    pub active: usize,
}

impl SelectionRange {
    pub fn new(anchor: usize, active: usize) -> Self {
        Self { anchor, active }
    }

    pub fn point(offset: usize) -> Self {
        Self {
            anchor: offset,
            active: offset,
        }
    }

    pub fn normalized(&self) -> (usize, usize) {
        if self.anchor <= self.active {
            (self.anchor, self.active)
        } else {
            (self.active, self.anchor)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }

    pub fn len(&self) -> usize {
        let (start, end) = self.normalized();
        end - start
    }

    pub fn collapse_to_active(&mut self) {
        self.anchor = self.active;
    }

    pub fn collapse_to_start(&mut self) {
        let (start, _) = self.normalized();
        self.anchor = start;
        self.active = start;
    }

    pub fn select_all(&mut self, len: usize) {
        self.anchor = 0;
        self.active = len;
    }
}
