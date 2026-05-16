#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRegion {
    pub start: f32,
    pub end: f32,
}
impl LoopRegion {
    pub fn snap(&mut self) {
        if self.end < self.start {
            std::mem::swap(&mut self.start, &mut self.end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_orders_region_bounds() {
        let mut region = LoopRegion {
            start: 4.0,
            end: 2.0,
        };

        region.snap();

        assert!(region.start <= region.end);
    }
}
