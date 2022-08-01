#![allow(dead_code)] // It's a fresh project and this isn't helpful
#![allow(mixed_script_confusables)] // Hell yeah, math!

use fermium::prelude::*;
use legion::*;
use ultraviolet::{Vec2, Vec3};

mod ecs;
mod gfx;
mod math;

use ecs::*;
use math::*;

fn poll_event() -> Option<SDL_Event> {
    let mut e = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut e) == 1 } {
        Some(e)
    } else {
        None
    }
}

pub mod color {
    use super::*;

    pub const WHITE: Vec3 = Vec3::new(0.84, 0.84, 0.84);

    pub const RED: Vec3 = Vec3::new(0.60, 0., 0.);
    pub const ORANGE: Vec3 = Vec3::new(0.84, 0.60, 0.);
    pub const GREEN: Vec3 = Vec3::new(0., 0.60, 0.);
    pub const YELLOW: Vec3 = Vec3::new(0.80, 0.80, 0.);

    pub const OHNO_PINK: Vec3 = Vec3::new(1., 0., 1.);
}

fn main() {
    let window_width: i32 = 500;
    let window_height: i32 = 750;

    let window = gfx::Window::new(window_width, window_height);
    let mut gpu = gfx::GpuDevice::new(&window);
    let mut world = World::default();

    // Shape of a brick & the paddle
    let dims = Vec2::new(5., 1.);

    // for the board
    let view_x = (dims.x + 1.) * 14. + 1.;
    let view_y = view_x * (window_height as f32 / window_width as f32);
    gpu.set_view(view_x, view_y);

    // (x, y) are position in the grid
    for y in 0..55 {
        let color: Vec3 = match y {
            0..=1 => color::RED,
            2..=3 => color::ORANGE,
            4..=5 => color::GREEN,
            6..=7 => color::YELLOW,
            _ => color::OHNO_PINK,
        };
        for x in 0..14 {
            // Note: Our x coordinate here must match the calculation for view_x above
            let pos_x = (dims.x + 1.) * (x as f32) + 1.;
            let pos_y = view_y - (dims.y + 1.) * (y as f32 + 1.);
            let pos = Vec2::new(pos_x, pos_y);

            world.push((
                Name(format!("Brick@({pos_x}, {pos_y})")),
                Position(pos),
                HitableQuad { dims },
                DrawableColoredQuad { dims, color },
                Breakable,
            ));
        }
    }

    // Add a user-controlled paddle
    let paddle_pos = Vec2::new(0.5 * view_x - dims.x / 2., 0.05 * view_y);
    let paddle_dims = dims;
    world.push((
        Name("Paddle".to_string()),
        Position(paddle_pos),
        HitableQuad { dims: paddle_dims },
        DrawableColoredQuad {
            dims: paddle_dims,
            color: color::WHITE,
        },
        Paddle,
    ));

    let mut ball_count = 0;

    // Spawn a starter ball
    ball_count += 1;
    let ball_pos = paddle_pos + Vec2::new(0.5 * paddle_dims.x - 0.5, 3. * paddle_dims.y);
    let ball_dims = Vec2::new(1., 1.);
    world.push((
        Name(format!("Ball-#{ball_count}")),
        Position(ball_pos),
        Velocity(Vec2::new(100., 35.)),
        HitableQuad { dims: ball_dims },
        DrawableColoredQuad {
            dims: ball_dims,
            color: color::WHITE,
        },
        Ball,
    ));

    // Hoisted out of the loop to reuse the allocation
    let mut quads = Vec::with_capacity(world.len());

    let mut paused = false;
    window.show();

    'main_loop: loop {
        // Handle events
        while let Some(e) = poll_event() {
            // Access to unions is unsafe, so this match block is going to get spicy
            let type_ = unsafe { e.type_ };
            match type_ {
                // Immediately quit everything - unhandled events are forever ignored
                SDL_QUIT => {
                    break 'main_loop;
                }

                SDL_KEYDOWN | SDL_KEYUP => {
                    let key = unsafe { e.key };

                    // Quit the app when "Q" is pressed
                    if key.keysym.sym == SDLK_q {
                        break 'main_loop;
                    }

                    // Toggle the simulation update when SPACE is pressed
                    if (type_ == SDL_KEYDOWN) && (key.repeat == 0) && (key.keysym.sym == SDLK_SPACE)
                    {
                        paused = !paused;
                    }

                    // Spawn a ball when "B" is pressed
                    if (type_ == SDL_KEYDOWN) && (key.keysym.sym == SDLK_b) {
                        let _ball = world.push((
                            Name(format!("Ball-#{ball_count}")),
                            Position(ball_pos),
                            Velocity(135. * random_direction()),
                            HitableQuad { dims: ball_dims },
                            DrawableColoredQuad {
                                dims: ball_dims,
                                color: color::WHITE,
                            },
                            Ball,
                        ));
                    }

                    // Clear all balls when "C" is pressed
                    if (type_ == SDL_KEYDOWN) && (key.repeat == 0) && (key.keysym.sym == SDLK_c) {
                        let mut query = <(Entity, &Ball)>::query();
                        let balls: Vec<_> = query.iter(&world).map(|(e, _)| *e).collect();
                        let ball_count = balls.len();
                        for ball in balls {
                            world.remove(ball);
                        }

                        println!("Removed {ball_count} balls");
                    }
                }

                // Ignore all other events
                _ => {}
            }
        }

        // Update gamestate
        const DELAY_MS: u32 = 5;
        let dt = (DELAY_MS as f32) * 1e-3;

        if !paused {
            {
                // Update ball positions
                {
                    let mut query = <(&mut Position, &Velocity)>::query();
                    for (Position(pos), Velocity(vel)) in query.iter_mut(&mut world) {
                        (*pos) += dt * *vel;
                    }
                }

                // Check if the ball is colliding with anything
                // Note: Balls do not interact with other balls
                {
                    let mut ball_query = <(Entity, &Position, &Velocity, &HitableQuad)>::query()
                        .filter(component::<Ball>());
                    let mut hitable_query =
                        <(Entity, &Position, &HitableQuad, Option<&Breakable>)>::query()
                            .filter(!component::<Ball>());

                    let mut breakables_hit = vec![];

                    for (_ball, Position(ball_pos), Velocity(ball_vel), ball_quad) in
                        ball_query.iter(&world)
                    {
                        let ball_aabb = Aabb::new_from_quad(*ball_pos, ball_quad.dims);

                        for (hitter, Position(pos), quad, maybe_breakable) in
                            hitable_query.iter(&world)
                        {
                            let aabb = Aabb::new_from_quad(*pos, quad.dims);

                            if let Some(_hit) =
                                ball_aabb.intersects_with_aabb_sweep(&aabb, dt * *ball_vel)
                            {
                                if maybe_breakable.is_some() {
                                    breakables_hit.push(*hitter);
                                }
                            }
                        }
                    }

                    // Remove anything that broke
                    for breakable in breakables_hit {
                        world.remove(breakable);
                    }

                    // Adjust position & velocity after the bounce
                    // TODO
                }
            }
        }

        // Render
        use gfx::shaders::PerQuad;

        {
            // Build quads for the renderer
            let mut query = <(&Position, &DrawableColoredQuad)>::query();
            for (Position(pos), drawable) in query.iter(&world) {
                quads.push(PerQuad {
                    pos: *pos,
                    dims: drawable.dims,
                    color: drawable.color,
                });
            }

            gpu.render_and_present(&quads);
            quads.clear();
        }

        // TODO: Better delay
        unsafe {
            SDL_Delay(DELAY_MS);
        }
    }

    unsafe {
        SDL_Quit();
    }
}
