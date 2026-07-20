//! Lightweight tweening animation system for AcmeUI Native.
#![forbid(unsafe_op_in_unsafe_fn)]

use slotmap::{SlotMap, new_key_type};

new_key_type! {
    /// Key for an active animation.
    pub struct AnimationKey;
}

/// Easing functions for animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

impl Easing {
    /// Apply the easing function to a normalized time `t` in [0, 1].
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Bounce => {
                // Standard CSS bounce ease-out
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

/// A tween animation between two f32 values.
#[derive(Clone, Debug)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub duration: std::time::Duration,
    pub elapsed: std::time::Duration,
    pub easing: Easing,
    pub delay: std::time::Duration,
    pub finished: bool,
    pub loop_count: u32, // 0 = infinite
    pub current_loop: u32,
    pub yoyo: bool,    // reverse direction each loop
    pub reverse: bool, // currently going backwards
}

impl Tween {
    pub fn new(from: f32, to: f32, duration_ms: u64) -> Self {
        Self {
            from,
            to,
            duration: std::time::Duration::from_millis(duration_ms),
            elapsed: std::time::Duration::ZERO,
            easing: Easing::Linear,
            delay: std::time::Duration::ZERO,
            finished: false,
            loop_count: 1,
            current_loop: 0,
            yoyo: false,
            reverse: false,
        }
    }

    /// Current interpolated value.
    pub fn value(&self) -> f32 {
        if self.finished {
            return if self.reverse { self.from } else { self.to };
        }
        let t = (self.elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0) as f32;
        let eased = self.easing.apply(t);
        self.from + (self.to - self.from) * eased
    }

    /// Normalized progress [0, 1].
    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0) as f32
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay = std::time::Duration::from_millis(ms);
        self
    }
    pub fn with_loop(mut self, count: u32) -> Self {
        self.loop_count = count;
        self
    }
    pub fn with_yoyo(mut self) -> Self {
        self.yoyo = true;
        self
    }
}

/// Manages all active animations.
#[derive(Default)]
pub struct AnimationEngine {
    animations: SlotMap<AnimationKey, Tween>,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tween animation and return its key.
    pub fn add(&mut self, tween: Tween) -> AnimationKey {
        self.animations.insert(tween)
    }

    /// Remove an animation by key.
    pub fn remove(&mut self, key: AnimationKey) -> bool {
        self.animations.remove(key).is_some()
    }

    /// Advance all animations by `dt`. Removes finished ones (unless looping).
    /// Returns list of keys that changed state (started, finished, or looped).
    pub fn update(&mut self, dt: std::time::Duration) -> Vec<AnimationUpdate> {
        let mut events = Vec::new();
        for (key, anim) in &mut self.animations {
            if anim.finished {
                continue;
            }

            // Handle delay
            if anim.elapsed.is_zero() && anim.delay > dt {
                anim.delay -= dt;
                continue;
            }

            let was_zero = anim.elapsed.is_zero();
            anim.elapsed += dt;

            if was_zero {
                events.push(AnimationUpdate::Started(key));
            }

            if anim.elapsed >= anim.duration {
                // Check looping
                if anim.loop_count == 0 || anim.current_loop + 1 < anim.loop_count {
                    anim.current_loop += 1;
                    anim.elapsed = std::time::Duration::ZERO;
                    if anim.yoyo {
                        anim.reverse = !anim.reverse;
                        std::mem::swap(&mut anim.from, &mut anim.to);
                    }
                    events.push(AnimationUpdate::Looped(key));
                } else {
                    anim.finished = true;
                    events.push(AnimationUpdate::Finished(key));
                }
            }
        }
        events
    }

    /// Get a reference to an animation.
    pub fn get(&self, key: AnimationKey) -> Option<&Tween> {
        self.animations.get(key)
    }

    /// Get a mutable reference to an animation.
    pub fn get_mut(&mut self, key: AnimationKey) -> Option<&mut Tween> {
        self.animations.get_mut(key)
    }

    /// Number of active animations.
    pub fn len(&self) -> usize {
        self.animations.len()
    }
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationUpdate {
    Started(AnimationKey),
    Looped(AnimationKey),
    Finished(AnimationKey),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tween_value_at_start() {
        let tween = Tween::new(0.0, 100.0, 1000);
        assert_eq!(tween.value(), 0.0);
        assert_eq!(tween.progress(), 0.0);
    }

    #[test]
    fn test_tween_value_at_half() {
        let mut tween = Tween::new(0.0, 100.0, 1000);
        tween.elapsed = std::time::Duration::from_millis(500);
        assert!((tween.value() - 50.0).abs() < 0.001);
        assert!((tween.progress() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_tween_value_at_end() {
        let mut tween = Tween::new(0.0, 100.0, 1000);
        tween.elapsed = std::time::Duration::from_millis(1000);
        assert!((tween.value() - 100.0).abs() < 0.001);
        assert!((tween.progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tween_finished_value() {
        let mut tween = Tween::new(0.0, 100.0, 1000);
        tween.elapsed = std::time::Duration::from_millis(2000);
        tween.finished = true;
        assert!((tween.value() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_linear() {
        assert!((Easing::Linear.apply(0.0) - 0.0).abs() < 0.001);
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 0.001);
        assert!((Easing::Linear.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_in() {
        assert!((Easing::EaseIn.apply(0.0) - 0.0).abs() < 0.001);
        assert!((Easing::EaseIn.apply(0.5) - 0.25).abs() < 0.001);
        assert!((Easing::EaseIn.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_out() {
        assert!((Easing::EaseOut.apply(0.0) - 0.0).abs() < 0.001);
        assert!((Easing::EaseOut.apply(0.5) - 0.75).abs() < 0.001);
        assert!((Easing::EaseOut.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_in_out() {
        assert!((Easing::EaseInOut.apply(0.0) - 0.0).abs() < 0.001);
        assert!((Easing::EaseInOut.apply(0.25) - 0.125).abs() < 0.001);
        assert!((Easing::EaseInOut.apply(0.5) - 0.5).abs() < 0.001);
        assert!((Easing::EaseInOut.apply(0.75) - 0.875).abs() < 0.001);
        assert!((Easing::EaseInOut.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_bounce() {
        // Bounce at boundaries
        assert!((Easing::Bounce.apply(0.0) - 0.0).abs() < 0.001);
        assert!((Easing::Bounce.apply(1.0) - 1.0).abs() < 0.001);
        // At t=0.5, should be > 0.5 (bounce effect)
        assert!(Easing::Bounce.apply(0.5) > 0.5);
    }

    #[test]
    fn test_tween_with_easing() {
        let tween = Tween::new(0.0, 100.0, 1000).with_easing(Easing::EaseIn);
        assert_eq!(tween.easing, Easing::EaseIn);
    }

    #[test]
    fn test_tween_with_delay() {
        let tween = Tween::new(0.0, 100.0, 1000).with_delay(500);
        assert_eq!(tween.delay, std::time::Duration::from_millis(500));
    }

    #[test]
    fn test_tween_with_loop() {
        let tween = Tween::new(0.0, 100.0, 1000).with_loop(3);
        assert_eq!(tween.loop_count, 3);
    }

    #[test]
    fn test_tween_with_yoyo() {
        let tween = Tween::new(0.0, 100.0, 1000).with_yoyo();
        assert!(tween.yoyo);
    }

    #[test]
    fn test_looping_animation_fires_events() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 100.0, 100).with_loop(3));

        let events = engine.update(std::time::Duration::from_millis(100));
        assert!(events.contains(&AnimationUpdate::Started(key)));
        assert!(events.contains(&AnimationUpdate::Looped(key)));
        assert!(!events.contains(&AnimationUpdate::Finished(key)));

        let events = engine.update(std::time::Duration::from_millis(100));
        assert!(events.contains(&AnimationUpdate::Looped(key)));

        let events = engine.update(std::time::Duration::from_millis(100));
        assert!(events.contains(&AnimationUpdate::Finished(key)));
    }

    #[test]
    fn test_yoyo_reverses_direction() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 100.0, 100).with_loop(2).with_yoyo());

        // First loop: 0 -> 100
        let _events = engine.update(std::time::Duration::from_millis(100));
        // After first loop, yoyo should swap from/to and set reverse
        let anim = engine.get(key).unwrap();
        assert!(anim.reverse, "yoyo should set reverse after first loop");
        assert!(
            (anim.from - 100.0).abs() < 0.001,
            "from should be swapped to 100"
        );
        assert!((anim.to - 0.0).abs() < 0.001, "to should be swapped to 0");

        // Second loop: 100 -> 0
        let _events = engine.update(std::time::Duration::from_millis(100));
        assert!(engine.get(key).unwrap().finished);
    }

    #[test]
    fn test_engine_tracks_active_animations() {
        let mut engine = AnimationEngine::new();
        assert!(engine.is_empty());
        assert_eq!(engine.len(), 0);

        let _key1 = engine.add(Tween::new(0.0, 1.0, 100));
        let _key2 = engine.add(Tween::new(0.0, 1.0, 200));
        assert_eq!(engine.len(), 2);
        assert!(!engine.is_empty());

        engine.update(std::time::Duration::from_millis(100));
        // One finishes, one continues
        assert_eq!(engine.len(), 2);
    }

    #[test]
    fn test_engine_get_and_get_mut() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 1.0, 100));

        let anim = engine.get(key).unwrap();
        assert!(!anim.finished);

        engine.get_mut(key).unwrap().loop_count = 5;
        assert_eq!(engine.get(key).unwrap().loop_count, 5);
    }

    #[test]
    fn test_engine_remove() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 1.0, 100));
        assert!(engine.remove(key));
        assert!(!engine.remove(key)); // already removed
        assert!(engine.is_empty());
    }

    #[test]
    fn test_engine_fires_started_event() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 100.0, 500));

        let events = engine.update(std::time::Duration::from_millis(1));
        assert!(events.contains(&AnimationUpdate::Started(key)));
    }

    #[test]
    fn test_engine_fires_finished_event() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 100.0, 100));

        let events = engine.update(std::time::Duration::from_millis(100));
        assert!(events.contains(&AnimationUpdate::Finished(key)));
    }

    #[test]
    fn test_delay_prevents_start() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 100.0, 100).with_delay(200));

        // Before delay expires
        let events = engine.update(std::time::Duration::from_millis(100));
        assert!(!events.contains(&AnimationUpdate::Started(key)));
        assert!(!engine.get(key).unwrap().finished);

        // After delay expires
        let events = engine.update(std::time::Duration::from_millis(100));
        assert!(events.contains(&AnimationUpdate::Started(key)));
    }

    #[test]
    fn test_infinite_loop() {
        let mut engine = AnimationEngine::new();
        let key = engine.add(Tween::new(0.0, 100.0, 50).with_loop(0)); // infinite

        for _ in 0..10 {
            let events = engine.update(std::time::Duration::from_millis(50));
            assert!(events.contains(&AnimationUpdate::Looped(key)));
            assert!(!engine.get(key).unwrap().finished);
        }
    }
}
