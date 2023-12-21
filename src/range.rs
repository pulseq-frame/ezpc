use std::ops::{Range, RangeFrom, RangeInclusive};

pub trait RangeArgument {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

impl RangeArgument for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end - 1
    }
}

impl RangeArgument for RangeFrom<usize> {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        usize::MAX
    }
}

impl RangeArgument for RangeInclusive<usize> {
    fn start(&self) -> usize {
        *self.start()
    }

    fn end(&self) -> usize {
        *self.end()
    }
}

impl RangeArgument for usize {
    fn start(&self) -> usize {
        *self
    }
    fn end(&self) -> usize {
        *self
    }
}
