pub enum EasingType {
    Linear,
    SmoothStep,
    QuadIn,
    QuadOut,
}

impl EasingType {
    /// Easing functions [0, 1] -> [0, 1], assumes linear input
    fn ease(&self, t: f32) -> f32 {
        match self {
            EasingType::Linear => t,
            EasingType::SmoothStep => {
                let t2 = t * t;
                -2f32 * t2 * t + 3f32 * t2
            }
            EasingType::QuadIn => t * t,
            EasingType::QuadOut => -t * t + 2f32 * t,
        }
    }
}

#[derive(Clone)]
pub enum TransitionTarget {
    Points { points: Vec<(i32, i32)> },
    Point { point: (i32, i32) },
}
impl TransitionTarget {
    /// Performs interpolation, after easing function
    fn interpolate(t: f32, start: &Self, end: &Self) -> Self {
        match (start, end) {
            (
                TransitionTarget::Points { points: start_points },
                TransitionTarget::Points { points: end_points },
            ) => {
                let new_points = start_points
                    .iter()
                    .zip(end_points.iter())
                    .map(|(&(sx, sy), &(ex, ey))| {
                        (
                            (sx as f32 * (1f32 - t) + ex as f32 * t) as i32,
                            (sy as f32 * (1f32 - t) + ey as f32 * t) as i32,
                        )
                    })
                    .collect();

                TransitionTarget::Points { points: new_points }
            }
            (
                TransitionTarget::Point { point: start_point },
                TransitionTarget::Point { point: end_point },
            ) => TransitionTarget::Point {
                point: (
                    (start_point.0 as f32 * (1f32 - t) + end_point.0 as f32 * t) as i32,
                    (start_point.1 as f32 * (1f32 - t) + end_point.1 as f32 * t) as i32,
                ),
            },
            _ => {
                panic!("Error: 'start' and 'end' parameters must be the same TransitionTarget type!")
            }
        }
    }
}

pub struct Transition {
    duration: f32,
    elapsed: f32,
    easing: EasingType,
    start_state: TransitionTarget,
    end_state: TransitionTarget,
    current_state: TransitionTarget,
}

impl Transition {
    /// Creates a transition
    pub fn initialize(
        easing: EasingType,
        duration: f32,
        start_state: TransitionTarget,
        end_state: TransitionTarget,
    ) -> Self {
        let current_state = start_state.clone();
        Transition {
            easing,
            duration,
            elapsed: 0.0,
            start_state,
            end_state,
            current_state,
        }
    }

    /// Main access point for the Transition : updates the progress based on delta time
    pub fn step(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let eased_t = self.easing.ease(t);
        self.current_state =
            TransitionTarget::interpolate(eased_t, &self.start_state, &self.end_state);
    }

    /// Returns true if the Transition is finished (elapsed time reached duration)
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Resets elpased time to 0
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Resets Transition and changes start and end targets
    pub fn reset_new(&mut self, new_start: TransitionTarget, new_end: TransitionTarget) {
        self.reset();

        self.start_state = new_start;
        self.end_state = new_end;
    }

    /// Returns the current state if target is points vec
    pub fn get_current_points(&self) -> &Vec<(i32, i32)> {
        match &self.current_state {
            TransitionTarget::Points { points } => { points }
            _ => {panic!("Error: called get_points() on transition with non-points target !")}
        }
    }

    /// Returns the start state if target is points vec
    pub fn get_start_points(&self) -> &Vec<(i32, i32)> {
        match &self.start_state {
            TransitionTarget::Points { points } => { points }
            _ => {panic!("Error: called get_points() on transition with non-points target !")}
        }
    }

    /// Returns the end state if target is points vec
    pub fn get_end_points(&self) -> &Vec<(i32, i32)> {
        match &self.end_state {
            TransitionTarget::Points { points } => { points }
            _ => {panic!("Error: called get_points() on transition with non-points target !")}
        }
    }

    /// Returns current state if target is single point
    pub fn get_current_point(&self) -> &(i32, i32) {
        match &self.current_state {
            TransitionTarget::Point { point } => { point }
            _ => {panic!("Error: called get_point() on transition with non-point target !")}
        }
    }
}