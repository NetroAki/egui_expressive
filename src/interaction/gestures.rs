/// Output of a tap gesture.
#[derive(Clone, Debug)]
pub struct TapEvent {
    pub pos: egui::Pos2,
    pub count: u32,
}

/// Output of a long-press gesture.
#[derive(Clone, Debug)]
pub struct LongPressEvent {
    pub pos: egui::Pos2,
}

/// Output of a swipe gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct SwipeEvent {
    pub direction: SwipeDirection,
    pub velocity: egui::Vec2,
}

/// Recognizes a tap (click without significant drag).
pub struct TapGesture {
    pub max_drag: f32,
    pub count: u32,
}

impl Default for TapGesture {
    fn default() -> Self {
        Self {
            max_drag: 5.0,
            count: 1,
        }
    }
}

impl TapGesture {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn double() -> Self {
        Self {
            count: 2,
            ..Default::default()
        }
    }
    pub fn max_drag(mut self, px: f32) -> Self {
        self.max_drag = px;
        self
    }

    pub fn recognize(&self, response: &egui::Response) -> Option<TapEvent> {
        let fired = if self.count == 2 {
            response.double_clicked()
        } else {
            response.clicked()
        };
        if fired && response.drag_delta().length() < self.max_drag {
            Some(TapEvent {
                pos: response
                    .interact_pointer_pos()
                    .unwrap_or(response.rect.center()),
                count: self.count,
            })
        } else {
            None
        }
    }
}

/// Recognizes a long press (pointer held without significant movement).
pub struct LongPressGesture {
    pub duration: f32,
    pub max_drag: f32,
}

impl Default for LongPressGesture {
    fn default() -> Self {
        Self {
            duration: 0.5,
            max_drag: 5.0,
        }
    }
}

impl LongPressGesture {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn duration(mut self, secs: f32) -> Self {
        self.duration = secs;
        self
    }

    pub fn recognize(
        &self,
        response: &egui::Response,
        ctx: &egui::Context,
        id: egui::Id,
    ) -> Option<LongPressEvent> {
        let press_start_id = id.with("__lp_start");
        let fired_id = id.with("__lp_fired");

        if response.is_pointer_button_down_on() && response.drag_delta().length() < self.max_drag {
            let start: f64 = ctx.data(|d| d.get_temp(press_start_id)).unwrap_or_else(|| {
                let t = ctx.input(|i| i.time);
                ctx.data_mut(|d| d.insert_temp(press_start_id, t));
                t
            });
            let elapsed = ctx.input(|i| i.time) - start;
            let already_fired: bool = ctx.data(|d| d.get_temp(fired_id)).unwrap_or(false);
            if elapsed >= self.duration as f64 && !already_fired {
                ctx.data_mut(|d| d.insert_temp(fired_id, true));
                return Some(LongPressEvent {
                    pos: response
                        .interact_pointer_pos()
                        .unwrap_or(response.rect.center()),
                });
            }
        } else {
            ctx.data_mut(|d| {
                d.remove::<f64>(press_start_id);
                d.remove::<bool>(fired_id);
            });
        }
        None
    }
}

/// Recognizes a swipe (fast drag in a cardinal direction).
pub struct SwipeGesture {
    pub min_velocity: f32,
    pub min_distance: f32,
}

impl Default for SwipeGesture {
    fn default() -> Self {
        Self {
            min_velocity: 200.0,
            min_distance: 30.0,
        }
    }
}

impl SwipeGesture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recognize(
        &self,
        response: &egui::Response,
        id: egui::Id,
        ctx: &egui::Context,
    ) -> Option<SwipeEvent> {
        let total_id = id.with("__sw_total");
        let start_time_id = id.with("__sw_start");

        if response.drag_started() {
            let now = ctx.input(|i| i.time);
            ctx.data_mut(|d| {
                d.insert_temp(start_time_id, now);
                d.insert_temp(total_id, egui::Vec2::ZERO);
            });
        }

        if response.dragged() {
            let prev: egui::Vec2 = ctx.data(|d| d.get_temp(total_id)).unwrap_or_default();
            ctx.data_mut(|d| d.insert_temp(total_id, prev + response.drag_delta()));
        }

        if response.drag_stopped() {
            let total: egui::Vec2 = ctx.data(|d| d.get_temp(total_id)).unwrap_or_default();
            let start_time: f64 = ctx.data(|d| d.get_temp(start_time_id)).unwrap_or(0.0);
            ctx.data_mut(|d| {
                d.remove::<egui::Vec2>(total_id);
                d.remove::<f64>(start_time_id);
            });

            let dist = total.length();
            if dist < self.min_distance {
                return None;
            }

            let now = ctx.input(|i| i.time);
            let duration = (now - start_time).max(0.016) as f32;
            let velocity = total / duration;
            if velocity.length() < self.min_velocity {
                return None;
            }

            let direction = if total.x.abs() > total.y.abs() {
                if total.x > 0.0 {
                    SwipeDirection::Right
                } else {
                    SwipeDirection::Left
                }
            } else if total.y > 0.0 {
                SwipeDirection::Down
            } else {
                SwipeDirection::Up
            };

            return Some(SwipeEvent {
                direction,
                velocity,
            });
        }
        None
    }
}
