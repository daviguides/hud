use super::spinners::{SPINNERS, SpinnerData};

/// The name of the animation a spinner falls back to when asked for one that does not exist.
const DEFAULT: &str = "dots";

/// The animation with this name, if there is one.
pub(crate) fn find(name: &str) -> Option<&'static SpinnerData> {
    SPINNERS.iter().find(|data| data.name == name)
}

/// The animation used when a name is not known: `dots`.
pub(crate) fn default_data() -> &'static SpinnerData {
    find(DEFAULT).unwrap_or(&SPINNERS[0])
}

/// The names of every animation, in alphabetical order.
pub(crate) fn names() -> impl Iterator<Item = &'static str> {
    SPINNERS.iter().map(|data| data.name)
}

/// The state of one spinner's animation: which frame it shows at a given moment. It follows
/// Rich's rule to the letter. The frame number is `((time - start) * speed) / (interval / 1000)`
/// plus the offset; the start is the time of the first frame asked for; a new speed takes effect
/// at the next frame, moving the offset to the current frame number and the start to now.
#[derive(Clone, Debug)]
pub(crate) struct Animation {
    data: &'static SpinnerData,
    speed: f64,
    start: Option<f64>,
    offset: f64,
    pending_speed: f64,
}

impl Animation {
    pub(crate) fn new(data: &'static SpinnerData, speed: f64) -> Animation {
        Animation {
            data,
            speed,
            start: None,
            offset: 0.0,
            pending_speed: 0.0,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.data.name
    }

    /// The same animation at another speed, from the start. Zero changes nothing.
    pub(crate) fn with_speed(mut self, speed: f64) -> Animation {
        if speed != 0.0 {
            self.speed = speed;
        }
        self
    }

    /// Asks for a new speed from the next frame on. Zero means no change.
    pub(crate) fn set_speed(&mut self, speed: f64) {
        if speed != 0.0 {
            self.pending_speed = speed;
        }
    }

    /// The frame shown at `time` (seconds on the spinner's clock).
    pub(crate) fn frame(&mut self, time: f64) -> &'static str {
        let start = *self.start.get_or_insert(time);
        let frame_no = ((time - start) * self.speed) / (self.data.interval / 1000.0) + self.offset;
        let count = self.data.frames.len() as i64;
        let index = (frame_no as i64).rem_euclid(count) as usize;
        let frame = self.data.frames[index];
        if self.pending_speed != 0.0 {
            self.offset = frame_no;
            self.start = Some(time);
            self.speed = self.pending_speed;
            self.pending_speed = 0.0;
        }
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dots(speed: f64) -> Animation {
        Animation::new(find("dots").unwrap(), speed)
    }

    #[test]
    fn the_first_frame_asked_for_fixes_the_start() {
        let mut animation = dots(1.0);
        assert_eq!(animation.frame(0.0), "⠋");
        assert_eq!(animation.frame(0.08), "⠙");
        assert_eq!(animation.frame(0.8), "⠋");
    }

    #[test]
    fn a_new_speed_takes_effect_at_the_next_frame_and_keeps_the_place() {
        let mut animation = dots(1.0);
        assert_eq!(animation.frame(0.0), "⠋");
        animation.set_speed(2.0);
        assert_eq!(animation.frame(0.16), "⠹");
        // frames last half as long from here, counted from the frame that was showing; the
        // number is 3.9999999999999996, which Rich truncates to 3 as well
        assert_eq!(animation.frame(0.24), "⠸");
    }

    #[test]
    fn there_are_73_animations_and_dots_is_the_default() {
        assert_eq!(names().count(), 73);
        assert_eq!(default_data().name, "dots");
        assert!(find("no such spinner").is_none());
    }
}
