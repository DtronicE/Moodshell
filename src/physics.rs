use crate::config::{FRICTION, GRAVITY, RESTITUTION};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub impact_frames: u8,
    pub walk_timer: f64,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            x: 20.0,
            y: 4.0,
            vx: 6.0,
            vy: 0.0,
            impact_frames: 0,
            walk_timer: 2.0,
        }
    }
}

impl PhysicsBody {
    pub fn jump(&mut self, impulse_y: f64, impulse_x: f64) {
        self.vy = -impulse_y.abs();
        self.vx += impulse_x;
    }

    pub fn update(
        &mut self,
        dt: f64,
        bounds_w: f64,
        bounds_h: f64,
        sprite_w: f64,
        focus_mode: bool,
    ) {
        if self.impact_frames > 0 {
            self.impact_frames -= 1;
        }

        let floor_y = (bounds_h - 1.0).max(1.0);
        let max_x = (bounds_w - sprite_w).max(1.0);

        // In Focus Mode, the companion settles gently on the floor and stays still
        if focus_mode {
            self.vx *= 0.8;
            self.vy += GRAVITY * dt;
            self.y += self.vy * dt;
            if self.y >= floor_y {
                self.y = floor_y;
                self.vy = 0.0;
            }
            self.x = self.x.clamp(1.0, max_x);
            return;
        }

        // Autonomous random walking & hopping impulses
        self.walk_timer -= dt;
        if self.walk_timer <= 0.0 {
            let mut rng = rand::thread_rng();
            self.walk_timer = rng.gen_range(1.8..4.5);

            // Only initiate a walk/hop when near the floor
            if (self.y - floor_y).abs() < 0.8 {
                let action: u8 = rng.gen_range(0..10);
                if action < 6 {
                    // Random walk step
                    self.vx = rng.gen_range(-14.0..14.0);
                } else if action < 9 {
                    // Playful hop
                    self.vy = rng.gen_range(-24.0..-15.0);
                    self.vx = rng.gen_range(-10.0..10.0);
                }
            }
        }

        // Apply gravity
        self.vy += GRAVITY * dt;

        // Integrate position
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Floor collision
        if self.y >= floor_y {
            self.y = floor_y;
            if self.vy.abs() > 14.0 {
                self.impact_frames = 5;
            }
            if self.vy.abs() > 2.5 {
                self.vy = -self.vy * RESTITUTION;
            } else {
                self.vy = 0.0;
            }
            self.vx *= FRICTION;
        }

        // Ceiling collision
        if self.y < 1.0 {
            self.y = 1.0;
            self.vy = self.vy.abs() * RESTITUTION;
        }

        // Left/Right wall collisions
        if self.x <= 1.0 {
            self.x = 1.0;
            if self.vx.abs() > 8.0 {
                self.impact_frames = 4;
            }
            self.vx = self.vx.abs() * RESTITUTION;
        } else if self.x >= max_x {
            self.x = max_x;
            if self.vx.abs() > 8.0 {
                self.impact_frames = 4;
            }
            self.vx = -self.vx.abs() * RESTITUTION;
        }
    }
}
