use std::f32::INFINITY;
use std::iter::repeat;

use crate::kapp::*;
use crate::kmath::*;
use crate::kimg::*;
use crate::level::*;
use crate::enemy_repo::*;
use crate::texture_buffer::TextureBuffer;
use std::time::SystemTime;
use std::time::Duration;


const PLAYER_SPEED: f32 = 0.1;
const PLAYER_RADIUS: f32 = 0.003;
const PLAYER_INVUL_TIME: f32 = 0.3;
const PLAYER_COLOUR_INNER: Vec4 = Vec4::grey(0.7);
const PLAYER_COLOUR_OUTER: Vec4 = Vec4::grey(0.0);

const LASER_DPS: f32 = 3.2;
const LASER_W: f32 = 0.003;

const BIBLE_DPS: f32 = 6.0;
const BIBLE_ORBIT_RADIUS: f32 = 0.023;
const BIBLE_SPEED: f32 = 7.0;
const BIBLE_SIZE: f32 = 0.006;
const BIBLE_GROW_SPEED: f32 = 0.5;

const FG_SAT: f32 = 0.8;
const BG_SAT: f32 = 0.6;
const FG_VAL: f32 = 0.15;
const BG_VAL: f32 = 0.4;

pub struct Game {
    seed: u32,
    frame: u64,
    t: f32,
    stale: bool,
    l: Level,
    camera: Rect,
    zoom: f32,

    player_hp: f32,
    player_damage_time: f32,
    player_pos: Vec2,
    player_bible_start: f32,
    player_bible_dir: bool,

    enemy_pos: Vec<Vec2>,
    enemy_v: Vec<Vec2>,
    enemy_hp: Vec<f32>,
    enemy_kill: Vec<bool>,
    enemy_type: Vec<usize>,
    enemy_last_attack: Vec<f32>,
    enemy_seed: Vec<u32>,

    enemies_pause: bool,
    repo: EnemyRepo
}

impl Game {
    fn advance_level(&mut self) {
        self.player_hp = 1.0;
        self.clear_enemies();
        self.stale = true;
        self.l.floor += 1;
        self.l.seed += 1;
        self.l.gen();
        'OUTER:
        for i in 0..self.l.w {
            for j in 0..self.l.h {
                if self.l.grid_type[j * self.l.w + i] == 2 {
                    (self.player_pos.x, self.player_pos.y) = self.l.cell_xy(i, j);
                    break 'OUTER;
                }
            }
        }

        let difficulty = self.l.floor as f32 / 10.0;
        println!(" advance level, {}", difficulty);

        // spawn enemies
        let sw = 25;
        let sh = 25;
        for i in 0..sw {
            for j in 0..sh {
                let si = khash2i(i, j, self.l.seed * 124891247);

                if chance(khash(self.seed * 1324147 + i as u32 * 124712547 + j as u32 * 131917), difficulty) {
                    let x = i as f32 / sw as f32;
                    let y = j as f32 / sh as f32;
                    
                    let x = x + 1.0/sw as f32 * krand(khash2i(i, j, self.l.seed));
                    let y = y + 1.0/sh as f32 * krand(khash2i(i, j, self.l.seed * 148971247));
                    
                    let pp = self.l.point(x, y);
                    if pp.gtype == STAIRS_DOWN { continue; }
                    let packdesc = self.repo.spawn_table[khash(si * 2312317) as usize % self.repo.spawn_table.len()].clone();
                    let pack_range = 0.04;

                    for (etype, qty) in packdesc {
                        for n in 0..qty {
                            let si = si + etype as u32 * 2131241477 + n as u32 * 21312377;
                            // get the point and then optionally reject
                            let dx = pack_range * (krand(si) - 0.5);
                            let dy = pack_range * (krand(si * 13123147) - 0.5);
                            let x = x + dx;
                            let y = y + dy;

                            let pp = self.l.point(x, y);

                            let er = self.repo.get(etype);
                            // spawn enemies
                            if !pp.walkable { continue; }
                            if Vec2::new(x, y).dist(self.player_pos) < er.acquisition_radius {
                                continue;
                            }

                            self.spawn_enemy(etype, er.initial_hp, Vec2::new(x, y), Vec2::zero(), khash(si));
                        }
                        
                    }
                }
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        let l = Level::default();
        let mut g = Game {
            frame: 0,
            t: 0.0,
            stale: true,
            l,
            player_pos: Vec2::new(0.0, 0.0),
            player_hp: 1.0,
            player_damage_time: -100.0,
            camera: Rect::new(0.0, 0.0, 1.0, 1.0),
            zoom: 0.15,
            enemy_pos: Vec::new(),
            enemy_kill: Vec::new(),
            enemy_hp: Vec::new(),
            enemy_type: Vec::new(),
            enemy_v: Vec::new(),
            enemy_seed: Vec::new(),
            enemy_last_attack: Vec::new(),
            enemies_pause: false,
            seed: SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_nanos(34123123)).subsec_nanos(),
            repo: EnemyRepo::default(),
            player_bible_start: 0.0,
            player_bible_dir: false,
        };
        g.advance_level();
        g
    }
}

impl Demo for Game {
    fn frame(&mut self, inputs: &FrameInputs, outputs: &mut FrameOutputs) {
        self.frame += 1;

        let mut dt = inputs.dt;
        if self.player_hp > 0.0 {
            self.t += dt;
        } else {
            dt = 0.0;
        }

        if inputs.key_pressed(VirtualKeyCode::P) {
            self.enemies_pause = !self.enemies_pause;
        }

        if inputs.key_pressed(VirtualKeyCode::R) {
            self.player_hp = 1.0;
            self.l.floor = 0;
            self.advance_level();
        }
        if inputs.key_pressed(VirtualKeyCode::Return) {
            if self.player_pos.dist(self.l.stairs_down) < 0.1 {
                self.advance_level();
            }
        }

        outputs.glyphs.push_str(&format!("Level {}", self.l.floor), 0.0, 0.0, 0.01, 0.01, 3.0, Vec4::new(1.0, 1.0, 1.0, 1.0));

        if self.stale {
            let hue = krand(inputs.seed) * 360.0;
            let fg = Vec4::new(hue, FG_SAT, FG_VAL, 1.0).hsv_to_rgb();
            let bg = Vec4::new(hue, BG_SAT, BG_VAL, 1.0).hsv_to_rgb();
            
            let w = 1000;
            let h = 1000;
            let mut tb = TextureBuffer::new(w, h);
            for i in 0..w {
                for j in 0..h {
                    let pp = self.l.point(i as f32 / w as f32, j as f32 / h as f32);
                    let c = if pp.walkable {
                        fg
                    } else {
                        bg
                    };
                    tb.set(i as i32, (h - j - 1) as i32, c);
                }
            }
            outputs.set_texture.push((tb, 0));
            self.stale = false;
        }

        let mut pv = Vec2::new(0.0, 0.0);
        if inputs.key_held(VirtualKeyCode::W) {
            pv.y -= 1.0;
        } else if inputs.key_held(VirtualKeyCode::S) {
            pv.y = 1.0;
        };
        if inputs.key_held(VirtualKeyCode::A) {
            pv.x -= 1.0;
        } else if inputs.key_held(VirtualKeyCode::D) {
            pv.x = 1.0;
        };
        pv = pv.normalize();

        let pspeed = dt * PLAYER_SPEED;
        let new_pos = self.player_pos + pspeed * pv;
        let pp = self.l.point(new_pos.x, new_pos.y);
        if pp.walkable {
            self.player_pos = new_pos;
        } else {
            let n = self.l.estimate_normal(new_pos, pspeed);
            if let Some(n) = n {
                self.player_pos = new_pos + n * pspeed;
            }
        }

        if !self.enemies_pause {
            // update enemies velocity
            for i in 0..self.enemy_pos.len() {
                let etype = self.enemy_type[i];
                let er = self.repo.get(etype);
                if er.is_projectile {
                    continue;
                }
                let wander_vec = Vec2::new(0.5 - noise1d(self.t, self.enemy_seed[i]), 0.5 - noise1d(self.t, self.enemy_seed[i] * 12390471));
                let mut will = Vec2::zero();
                if self.enemy_pos[i].dist(self.player_pos) < er.acquisition_radius {
                    will = (self.player_pos - self.enemy_pos[i]).normalize();
                }
                self.enemy_v[i] = will * er.speed_to_target + er.speed_wander * wander_vec.normalize();
            }
            
            // move enemy
            for i in 0..self.enemy_pos.len() {
                let etype = self.enemy_type[i];
                let er = self.repo.get(etype);
                // do rest of stuff
                
                let move_vec = self.enemy_v[i] * dt;
                
                let espeed = move_vec.magnitude();
                let new_pos = self.enemy_pos[i] + move_vec;
                

                // collide with terrain
                let pp = self.l.point(new_pos.x, new_pos.y);
                if pp.walkable {
                    self.enemy_pos[i] = new_pos;
                } else {
                    if er.is_projectile {
                        self.enemy_kill[i] = true;
                    } else {
                        let n = self.l.estimate_normal(new_pos, espeed);
                        if let Some(n) = n {
                            self.enemy_pos[i] = new_pos + n * espeed;
                        }
                    }
                }
            }
        }

        // collide enemies and players       
        for i in 0..self.enemy_pos.len() {
            let v = self.enemy_pos[i]-self.player_pos;
            let dist = v.magnitude();
            let etype = self.enemy_type[i];
            let er = self.repo.get(etype);
            let overlap = PLAYER_RADIUS + er.radius - dist;
            if overlap > 0.0 {
                self.enemy_pos[i] = self.enemy_pos[i] + 0.5 * v.normalize() * overlap;
                self.player_pos = self.player_pos - 0.5 * v.normalize() * overlap;
                if er.is_projectile {
                    self.enemy_kill[i] = true;
                }
                if self.t - self.player_damage_time > PLAYER_INVUL_TIME {
                    self.player_hp -= er.melee_damage;
                    self.player_damage_time = self.t;
                }
            }
            
        }

        // collide enemies
        for i in 0..self.enemy_pos.len() {
            let etypei = self.enemy_type[i];
            let eri = self.repo.get(etypei);

            if eri.is_projectile {
                continue;
            }
            for j in 0..self.enemy_pos.len() {
                if i == j {continue;}
                let etypej = self.enemy_type[j];
                let erj = self.repo.get(etypej);
                if erj.is_projectile {
                    continue;
                }
                let v = self.enemy_pos[i]-self.enemy_pos[j];
                let dist = v.magnitude();
                let overlap = eri.radius + erj.radius - dist;
                if overlap > 0.0 {
                    self.enemy_pos[i] = self.enemy_pos[i] + 0.5 * v.normalize() * overlap;
                    self.enemy_pos[j] = self.enemy_pos[j] - 0.5 * v.normalize() * overlap;
                }
            }
        }

        // shoot projectiles
        for i in 0..self.enemy_type.len() {
            let etype = self.enemy_type[i];
            let er = self.repo.get(etype);
            let u = self.player_pos - self.enemy_pos[i];
            if er.projectile > -1 && u.magnitude() < er.shoot_range && (self.t - self.enemy_last_attack[i]) > er.projectile_cooldown {   // & cast a ray to see if theres vision
                let mut dir = u.normalize();
                if etype == 9 {
                    dir = Vec2::new(dir.y, dir.x);
                }
                let spawn_etype = er.projectile as usize;
                let ser = self.repo.get(spawn_etype);
                let si = self.enemy_seed[i];
                self.spawn_enemy(spawn_etype, ser.initial_hp, self.enemy_pos[i], dir * ser.speed_to_target, si);
                self.enemy_last_attack[i] = self.t;
            }
        }
        
        self.camera = Rect::new_centered(self.player_pos.x, self.player_pos.y, self.zoom * inputs.screen_rect.aspect(), self.zoom);
        let mouse_world = inputs.mouse_pos.transform(inputs.screen_rect, self.camera);
        let cam_center = self.player_pos.lerp(mouse_world, 0.2);
        self.camera = Rect::new_centered(cam_center.x, cam_center.y, self.zoom * inputs.screen_rect.aspect(), self.zoom);
        
        let r = self.camera.pseudo_inverse();
        let p_screen_pos = self.player_pos.transform(Rect::unit(), r);
        
        if inputs.lmb == KeyStatus::Pressed && self.player_hp > 0.0 {
            let rising = inputs.lmb == KeyStatus::JustPressed;
            let falling = inputs.lmb == KeyStatus::JustReleased;
            self.do_item(inputs, outputs, 0, dt, rising, falling);
        }

        if inputs.rmb == KeyStatus::Pressed || inputs.rmb == KeyStatus::JustPressed && self.player_hp > 0.0 {
            let rising = inputs.rmb == KeyStatus::JustPressed;
            let falling = inputs.rmb == KeyStatus::JustReleased;
            self.do_item(inputs, outputs, 1, dt, rising, falling);
        }
        


        outputs.draw_texture.push((r, 0, 1.0));

        let p_radius = PLAYER_RADIUS * r.h;
        outputs.canvas.put_circle(p_screen_pos, p_radius * 1.2, 1.5, PLAYER_COLOUR_OUTER);
        let player_colour = if self.t - self.player_damage_time < PLAYER_INVUL_TIME {
            Vec4::new(1.0, 1.0, 1.0, 1.0)
        } else {
            PLAYER_COLOUR_INNER
        };
        outputs.canvas.put_circle(p_screen_pos, p_radius * 1.0, 1.6, player_colour);

        for i in 0..self.enemy_pos.len() {
            let ep_screen = self.enemy_pos[i].transform(Rect::unit(), r);
            let etype = self.enemy_type[i];
            let er = self.repo.get(etype);
            let e_radius = er.radius * r.h;
            outputs.canvas.put_circle(ep_screen, e_radius * 1.2, 1.5, er.colour_outer);
            outputs.canvas.put_circle(ep_screen, e_radius * 1.0, 1.6, er.colour_inner);

        }

        let player_health_rect = Rect::new(0.1, 0.8, 0.1, 0.1).dilate_pc(-0.2);
        outputs.canvas.put_rect(player_health_rect.dilate_pc(0.1), 3.0, Vec4::grey(0.0));
        outputs.canvas.put_rect(player_health_rect.child(0.0, 1.0 - self.player_hp.max(0.0), 1.0, self.player_hp.max(0.0)), 3.1, Vec4::new(1.0, 0.0, 0.0, 1.0));

        // stairs up
        let s = r.h * 0.02;
        let stairs_up_rect = self.l.stairs_up.transform(Rect::unit(), r).rect_centered(s, s);
        let stair_colour = Vec4::grey(0.4);
        outputs.canvas.put_rect(stairs_up_rect.child(0.0, 0.0, 0.33, 1.0), 1.3, stair_colour);
        outputs.canvas.put_rect(stairs_up_rect.child(0.33, 0.33, 0.33, 1.0 - 0.33), 1.3, stair_colour);
        outputs.canvas.put_rect(stairs_up_rect.child(0.66, 0.66, 1.0 - 0.66, 1.0 - 0.66), 1.3, stair_colour);
        
        // stairs down
        let stairs_down_rect = self.l.stairs_down.transform(Rect::unit(), r).rect_centered(s, s).child(0.0, 0.0, 1.0, 0.8);
        outputs.canvas.put_rect(stairs_down_rect, 1.1, stair_colour);
        let stairs_down_rect = stairs_down_rect.dilate_pc(-0.1);
        outputs.canvas.put_rect(stairs_down_rect, 1.2, Vec4::new(0.0, 0.0, 0.0, 1.0));
        let stairs_down_rect = stairs_down_rect.child(0.0, 0.2, 1.0, 0.8);
        let stair_colour = Vec4::grey(0.3);
        
        outputs.canvas.put_rect(stairs_down_rect.child(0.0, 0.0, 0.33, 1.0), 1.3, stair_colour);
        outputs.canvas.put_rect(stairs_down_rect.child(0.33, 0.33, 0.33, 1.0 - 0.33), 1.3, stair_colour);
        outputs.canvas.put_rect(stairs_down_rect.child(0.66, 0.66, 1.0 - 0.66, 1.0 - 0.66), 1.3, stair_colour);


        // // vignette
        // let vw = 100;
        // let vh = 100;
        // let mut tb = TextureBuffer::new(vw, vh);

        // let targets = 
        //     (0..vw).zip(repeat(0).take(vw)).chain(
        //     (0..vw).zip(repeat(vh - 1).take(vw)).chain(
        //     repeat(0).zip(0..vh).chain(
        //     repeat(vw - 1).zip(0..vh))));

        // for (targ_i, targ_j) in targets {
        //         let p = Vec2::new(i as f32 / vw as f32, j as f32 / vh as f32);
        //         let world_pos = p.transform(Rect::unit(), self.camera);
        //         // conduct raycast
        //         let stride = 0.01;
        //         let mut colour = Vec4::new(0.0, 0.0, 0.0, 0.0);
        //         let mut d = 0.00;
        //         let u = world_pos - self.player_pos;
        //         loop {
        //             let pos = self.player_pos + d * u.normalize();
        //             d += stride;
        //             let pp = self.l.point(pos.x, pos.y);
        //             if !pp.walkable {
        //                 colour = Vec4::new(0.0, 0.0, 0.0, 0.9);
        //                 break;
        //             }
        //             if d >= u.magnitude() {
        //                 break;
        //             }
        //         }
        //         tb.set(i as i32, vh as i32 - j as i32 - 1, colour)
        //     }
        // }
        // outputs.set_texture.push((tb, 1));
        // outputs.draw_texture.push((inputs.screen_rect, 1, 2.5));

        // vignette
        // oh we are going to have fun
        let vw = 400;
        let vh = 400;


        let mut mask_vals = vec![0.0; vw * vh];
        // now set all to black if it fails raycast
        // then erode / soften

        for i in 0..vw {
            for j in 0..vh {
                let x = (0.5 + i as f32) / vw as f32;
                let y = (0.5 + j as f32) / vh as f32;
                let p = Vec2::new(x as f32, y);
                let world_pos = p.transform(Rect::unit(), self.camera);
                
                let d = (world_pos-self.player_pos).magnitude();

                let outerd = noise1d(self.t + 0.41, self.seed * 141971237) - 0.5;
                if (self.l.ray_intersects_wall(self.player_pos, world_pos).is_some() && d > 0.01) || d > 0.12 + outerd * 0.01 {
                    mask_vals[j*vw + i] = 1.0;
                } else {
                    mask_vals[j*vw + i] = (5.0*d).min(1.0);
                }
            }
        }

        for _erosion in 0..6 {
            let mut mask_vals_new = mask_vals.clone();
            for i in 0..vw {
                for j in 0..vh {
                    // if any 4neighbour has any alpha we divide by 2
                    let dx = [-1, 0, 1, 0];
                    let dy = [0, -1, 0, 1];

                    
                    for n in 0..4 {
                        let nx = i as i32 + dx[n];
                        let ny = j as i32 + dy[n];
                        if nx < 0 { continue; }
                        if ny < 0 { continue; }
                        if nx > vw as i32 - 1 { continue; }
                        if ny > vh as i32 - 1 { continue; }
                        if mask_vals[vw * ny as usize + nx as usize] < 1.0 {
                            mask_vals_new[vw * j + i] *= 0.9;
                            break;
                        }
                    }
                }
            }
            mask_vals = mask_vals_new;
        }


        let mut tb = TextureBuffer::new(vw, vh);

        for i in 0..vw {
            for j in 0..vh {
                tb.set(i as i32, vh as i32 - j as i32 - 1, Vec4::new(0.0, 0.0, 0.0, mask_vals[j*vw + i]));
            }
        }
        outputs.set_texture.push((tb, 1));
        outputs.draw_texture.push((inputs.screen_rect, 1, 2.5));


        // let d_mouse_world = self.l.wall_distance(mouse_world);
        // outputs.canvas.put_circle(inputs.mouse_pos, d_mouse_world * r.h, 1.0, Vec4::new(0.7, 0.4, 0.0, 1.0));


        self.cull_enemies();
    }
}

// for collision
// proper way is probably sdf or something. so like SDWalkable, and you can combine with min and max etc

impl Game {
    pub fn clear_enemies(&mut self) {
        self.enemy_hp = Vec::new();
        self.enemy_v = Vec::new();
        self.enemy_pos = Vec::new();
        self.enemy_kill = Vec::new();
        self.enemy_last_attack = Vec::new();
        self.enemy_type = Vec::new();
        self.enemy_seed = Vec::new();
    }

    pub fn cull_enemies(&mut self) {
        let mut i = self.enemy_kill.len();
        loop {
            if i == 0 {
                break;
            }
            i -= 1;
            if self.enemy_kill[i] || self.enemy_hp[i] < 0.0 {
                self.enemy_kill.swap_remove(i);
                self.enemy_pos.swap_remove(i);
                self.enemy_hp.swap_remove(i);
                self.enemy_type.swap_remove(i);
                self.enemy_v.swap_remove(i);
                self.enemy_last_attack.swap_remove(i);
                self.enemy_seed.swap_remove(i);
            }
        }
    }

    pub fn spawn_enemy(&mut self, etype: usize, hp: f32, pos: Vec2, v: Vec2, seed: u32) {
        self.enemy_kill.push(false);
        self.enemy_v.push(v);
        self.enemy_type.push(etype);
        self.enemy_hp.push(hp);
        self.enemy_last_attack.push(-100.0);
        self.enemy_pos.push(pos);
        self.enemy_seed.push(seed)
    }
}

impl Game {
    pub fn do_item(&mut self, inputs: &FrameInputs, outputs: &mut FrameOutputs, id: u32, dt: f32, rising: bool, falling: bool) {
        let mouse_world = inputs.mouse_pos.transform(inputs.screen_rect, self.camera);
        let r = self.camera.pseudo_inverse();
        let p_screen_pos = self.player_pos.transform(Rect::unit(), r);

        if id == 0 {    // laser
            let laser_dir = (mouse_world - self.player_pos).normalize();
            // let mut stride = 0.01;
            let mut laser_t = 0.0;
            loop {
                let p = self.player_pos + (laser_t) * laser_dir;

                let dist = self.l.wall_distance(p);
                if dist < 0.0001 {
                    break;
                }
                laser_t += dist;

                // let pp = self.l.point(p.x, p.y);
                // if pp.walkable {
                //     laser_t += self.l.wall_distance(p);
                //     // laser_t += stride;
                // } else {
                //     stride /= 2.0;
                // }
                // if stride <= 0.0001 {
                //     break;
                // }
            }

            let mut nearest_enemy_t = INFINITY;
            let mut nearest_enemy_id: Option<usize> = None;
            for i in 0..self.enemy_pos.len() {
                let etype = self.enemy_type[i];
                let er = self.repo.get(etype);
                if er.is_projectile { continue; }
                let player_to_enemy = self.enemy_pos[i] - self.player_pos;
                let x = player_to_enemy.dot(laser_dir);
                if x < 0.0 { continue; }
                let proj = x*laser_dir;
                let laser_to_enemy = player_to_enemy - proj;
                
                if laser_to_enemy.magnitude() < LASER_W + er.radius && player_to_enemy.magnitude() < laser_t {
                    nearest_enemy_t = player_to_enemy.magnitude();
                    nearest_enemy_id = Some(i);
                }
            }
            if let Some(laser_enemy_id) = nearest_enemy_id {
                laser_t = nearest_enemy_t;
                self.enemy_hp[laser_enemy_id] -= dt * LASER_DPS;
            }

            outputs.canvas.put_line(p_screen_pos, p_screen_pos + r.h * laser_t * laser_dir, LASER_W * r.h, 1.4, Vec4::new(1.0, 0.0, 0.0, 1.0));
        } else if id == 1 { // bible
            if rising {
                self.player_bible_start = self.t;
                self.player_bible_dir = !self.player_bible_dir;
            }
            let t_bible = self.t - self.player_bible_start;
            let radius_multiplier = (t_bible*BIBLE_GROW_SPEED).min(1.0);
            let bible_radius = radius_multiplier * BIBLE_ORBIT_RADIUS;
            let mut bible_phase = (t_bible * BIBLE_SPEED) % (2.0 * PI);
            if self.player_bible_dir {
                bible_phase *= -1.0;
            }
            for i in 0..self.enemy_pos.len() {
                let bp1 = self.player_pos.offset_r_theta(bible_radius, bible_phase);
                let bp2 = self.player_pos.offset_r_theta(bible_radius, bible_phase - PI);
                let mindist = self.enemy_pos[i].dist(bp1).min(self.enemy_pos[i].dist(bp2));
                let er = self.repo.get(self.enemy_type[i]);
                let hit = mindist < BIBLE_SIZE + er.radius;
                if hit {
                    self.enemy_hp[i] -= dt * BIBLE_DPS;
                }
                let bp1 = bp1.transform(Rect::unit(), r);
                let bp2 = bp2.transform(Rect::unit(), r);
                outputs.canvas.put_rect(bp1.rect_centered(BIBLE_SIZE * 1.5 * r.h, BIBLE_SIZE * 1.5 * r.h), 1.5, Vec4::new(0.0, 0.0, 1.0, 1.0));
                outputs.canvas.put_rect(bp2.rect_centered(BIBLE_SIZE * 1.5 * r.h, BIBLE_SIZE * 1.5 * r.h), 1.5, Vec4::new(0.0, 0.0, 1.0, 1.0));
            }
        } else if id == 2 { // giant sword

        }
    }
}


// definitely pre compute a distance field and flood fill it for raymarching