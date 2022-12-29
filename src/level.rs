
use crate::kmath::*;
use crate::priority_queue::*;
use crate::distance_field::*;
use ordered_float::OrderedFloat;
use itertools::Itertools;
use std::f32::INFINITY;
use std::time::SystemTime;
use std::time::Duration;
use std::collections::VecDeque;


pub struct PointProperties {
    pub ni: usize,
    pub nj: usize,
    pub d: f32,
    pub walkable: bool,
    pub gtype: u32,
}
pub struct Level {
    pub seed: u32,
    pub floor: i32,
    pub w: usize,
    pub h: usize,
    pub grid_type: Vec<u32>,
    pub stairs_up: Vec2,
    pub stairs_down: Vec2,

    pub distances: Vec<f32>,
    pub walldirs: Vec<Vec2>,
    pub dw: usize,
    pub dh: usize,
}

pub const STAIRS_DOWN: u32 = 2;
pub const STAIRS_UP: u32 = 3;
pub const ENEMY_SITE: u32 = 1;
pub const CLOSED: u32 = 0;

impl Level {
    pub fn cell_seed(&self, x: usize, y: usize) -> u32 {
        self.seed.wrapping_add(12312547u32.wrapping_mul(x as u32).wrapping_add(568812347u32.wrapping_mul(y as u32)))
    }
    pub fn cell_xy(&self, x: usize, y: usize) -> (f32, f32) {
        let s = self.cell_seed(x, y);

        (x as f32 / self.w as f32 + 1.0 / self.w as f32 * krand(s),
        y as f32 / self.h as f32 + 1.0 / self.h as f32 * krand(241231247u32.wrapping_mul(s)))
    }
    pub fn gen(&mut self) {
        let num_sites = 15;

        self.grid_type = vec![0; self.w*self.h];
        let mut candidates: Vec<(usize, usize, u32)> = (0..self.w).cartesian_product(0..self.h)
            .map(|(i, j)| (i, j, khash(self.cell_seed(i, j))))
            .filter(|(i, j, _)| *i != 0 && *j != 0 && *i < self.w-1 && *j < self.h-1)
            .collect();
        candidates.sort_by_key(|x| x.2);
        candidates.truncate(num_sites);
        for (i, j, _) in candidates.iter() {
            self.grid_type[j * self.w + i] = 1; // site
        }

        // find furthest site from candidates[0]
        let mut furthest_x = 0;
        let mut furthest_y = 0;
        let mut furthest_d2 = 0.0f32;
        let (x0, y0) = self.cell_xy(candidates[0].0, candidates[0].1);
        for i in 1..num_sites {
            let (x1, y1) = self.cell_xy(candidates[i].0, candidates[i].1);
            let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
            if d2 > furthest_d2 {
                furthest_d2 = d2;
                furthest_x = candidates[i].0;
                furthest_y = candidates[i].1;
            }
        }

        // stairs up at furthest_x, furthest_y
        self.grid_type[furthest_y * self.w + furthest_x] = 2;
        (self.stairs_up.x, self.stairs_up.y) = self.cell_xy(furthest_x, furthest_y);
        
        // find furthest site from that and make it stairs down
        let mut furthest_d2 = 0.0f32;
        let (x0, y0) = self.cell_xy(furthest_x, furthest_y);
        for i in 0..num_sites {
            let (x1, y1) = self.cell_xy(candidates[i].0, candidates[i].1);
            let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
            if d2 > furthest_d2 {
                furthest_d2 = d2;
                furthest_x = candidates[i].0;
                furthest_y = candidates[i].1;
            }            
        }
        self.grid_type[furthest_y * self.w + furthest_x] = 3;
        (self.stairs_down.x, self.stairs_down.y) = self.cell_xy(furthest_x, furthest_y);
        
        self.gen_distances()
    }

    pub fn gen_distances(&mut self) {
        self.dw = 1600;
        self.dh = 1600;
        let f = |x, y| self.point(x, y).walkable;
        (self.distances, self.walldirs) = gen_distance_field_sep(f, self.dw, self.dh);
    }

    pub fn point(&self, x: f32, y: f32) -> PointProperties {
        if x < 0.0 || y < 0.0 || x > 1.0 || y > 1.0 {
            return PointProperties {
                ni: 0,
                nj: 0,
                d: INFINITY,
                walkable: false,
                gtype: 0,
            }
        }

        let x_orig = x;
        let y_orig = y;

        let x = x + noise2d(x_orig * 64.0, y_orig * 64.0, self.seed.wrapping_mul(12349417)) * 0.01;
        let y = y + noise2d(x_orig * 64.0, y_orig * 64.0, self.seed.wrapping_mul(98341247)) * 0.01;

        let thickness = noise2d(x * 4.0, y * 4.0, self.seed.wrapping_mul(141471747)) * 0.03 + 0.005;

        let i_cell = (x * self.w as f32).floor() as i32;
        let j_cell = (y * self.h as f32).floor() as i32;

        let mut candidates: Vec<(usize, usize, f32)> = (-1..=1).cartesian_product(-1..=1)
            .map(|(i, j)| (i + i_cell, j + j_cell))
            .filter(|(i, j)| *i >= 0 && *j >= 0 && *i < self.w as i32 && *j < self.h as i32)
            .map(|(i, j)| (i as usize, j as usize))
            .map(|(i, j)| {
                let (x1, y1) = self.cell_xy(i, j);
                let d2 = (x - x1)*(x - x1) + (y - y1)*(y - y1);
                (i, j, d2.sqrt())
            })
            .collect();

        candidates.sort_by_key(|x| OrderedFloat(x.2));

        let gtype = self.grid_type[candidates[0].1 * self.w + candidates[0].0];

        let on_line = candidates.len() >= 2 && (candidates[0].2 - candidates[1].2).abs() < thickness;
        let open_cell = gtype > 0;

        let outer_line = candidates.len() >= 2 && (candidates[0].0 == 0 || candidates[0].0 == self.w-1 || candidates[0].1 == 0 || candidates[0].1 == self.h-1) &&
            (candidates[1].0 == 0 || candidates[1].0 == self.w-1 || candidates[1].1 == 0 || candidates[1].1 == self.h-1);
            

        let obstructions = noise2d(x_orig * 32.0, y_orig * 32.0, self.seed.wrapping_mul(123412157)) > 0.8;
        let obs_mask = noise2d(x_orig * 8.0, y_orig * 8.0, self.seed.wrapping_mul(10968547)) > 0.8;

        let walkable = (!outer_line && on_line) || (open_cell && !(obstructions && obs_mask));
            

        PointProperties {
            ni: candidates[0].0,
            nj: candidates[0].1,
            d: candidates[0].2,
            walkable,
            gtype,
        }
    }

    pub fn wall_distance(&self, p: Vec2) -> f32 {
        if p.x < 0.0 || p.y < 0.0 || p.x > 1.0 || p.y > 1.0 {
            return 0.0;
        }
        let xf = (self.dw-1) as f32 * p.x;
        let yf = (self.dh-1) as f32 * p.y;

        let i = xf.floor() as usize;
        let j = yf.floor() as usize;

        let d1 = self.distances[(j + 0) * self.dw + (i + 0)];
        let d2 = self.distances[(j + 0) * self.dw + (i + 1)];
        let d3 = self.distances[(j + 1) * self.dw + (i + 0)];  // panic here because we are looking outside the distance field // p: Vec2 { x: 0.0, y: 0.9975 } // yea get a few panics here hey
        let d4 = self.distances[(j + 1) * self.dw + (i + 1)];

        d1.min(d2.min(d3.min(d4)))
    }

    pub fn wall_dir(&self, p: Vec2) -> Vec2 {
        if p.x < 0.0 || p.y < 0.0 || p.x > 1.0 || p.y > 1.0 {
            return Vec2::zero();
        }
        let xf = (self.dw-1) as f32 * p.x;
        let yf = (self.dh-1) as f32 * p.y;

        let i = xf.floor() as usize;
        let j = yf.floor() as usize;

        let d1 = self.walldirs[(j + 0) * self.dw + (i + 0)];
        let d2 = self.walldirs[(j + 0) * self.dw + (i + 1)];
        let d3 = self.walldirs[(j + 1) * self.dw + (i + 0)];
        let d4 = self.walldirs[(j + 1) * self.dw + (i + 1)];

        let xfrac = xf.fract();
        let yfrac = yf.fract();

        d1.lerp(d2, xfrac).lerp(d3.lerp(d4, xfrac), yfrac)
    }

    // if we supply d threshold we can ray march with a certain clearance
    // maybe want to march by a bit less than d
    // we can probably handle edge of map and minimum res
    // enforce safe
    // and dont test past the end of the ray
    pub fn ray_intersects_wall(&self, p1: Vec2, p2: Vec2) -> Option<f32> {
        let mut acc = 0.0;
        let u = p2 - p1;
        let umag = u.magnitude();
        let udir = u.normalize();
        loop {
            if acc >= umag {
                return None
            }
            let p = p1 + acc * udir;
            let d = self.wall_distance(p);
            if d <= 0.0 {
                return Some(acc);
            }
            acc += d;
        }
    }

    pub fn collide_circle(&self, p: Vec2, r: f32) -> Option<Vec2> {
        let d = self.wall_distance(p);
        let dir = self.wall_dir(p);
        let overlap = r - d;
        if overlap < 0.0 {
            return None;
        }
        return Some(dir * overlap)
    }

    // ex: Vec<f32>,

    // should be able to estimate normals better, or get rid of the radius or something
    // I could have a lookup for that too

    // sdf for terrain should be like height is -1 to 1

    pub fn estimate_normal(&self, p: Vec2, r: f32) -> Option<Vec2> {
        const n_points: usize = 8;
        let mut count = 0;
        let mut offset_vecs = [Vec2::new(0.0, 0.0); 8];
        let mut walkable = [false; n_points];
        for i in 0..8 {
            let theta = i as f32 * 2.0 * PI / n_points as f32;
            offset_vecs[i] = r * Vec2::new(theta.cos(), theta.sin());
            let sv = offset_vecs[i] + p;
            let pp = self.point(sv.x, sv.y);
            if pp.walkable {
                walkable[i] = true;
                count += 1;
            }
        }
        if count == 0 {
            return None;
        } else {
            let mut acc = Vec2::new(0.0, 0.0);
            for i in 0.. 8 {
                if walkable[i] {
                    acc = acc + offset_vecs[i];
                }
            }
            return Some((acc / count as f32).normalize());
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        let w = 8;
        let h = 8;
        let mut l = Level {
            seed: SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_nanos(34123123)).subsec_nanos(),
            w,
            h,
            floor: 0,
            grid_type: vec![],
            stairs_up: Vec2::zero(),
            stairs_down: Vec2::zero(),
            distances: vec![],
            walldirs: vec![],
            dw: 0,
            dh: 0,
        };
        l.gen();
        l
    }
}

#[test]
fn test_level() {
    use crate::kimg::ImageBuffer;

    let l = Level::default();
    let mut im = ImageBuffer::new(400, 400);
    for i in 0..im.w {
        for j in 0..im.h {
            let x = i as f32 / im.w as f32;
            let y = j as f32 / im.h as f32;
            let pp = l.point(x, y);

            let mut c = (0, 0, 0);
            if pp.walkable {
                c = (255, 255, 255)
            }
            if l.grid_type[pp.nj * l.w + pp.ni] == 3 {
                if pp.d < 0.01 {
                    c = (0, 0, 255)
                }
            }
            if l.grid_type[pp.nj * l.w + pp.ni] == 2 {
                if pp.d < 0.01 {
                    c = (0, 255, 0)
                }
            }
            if l.grid_type[pp.nj * l.w + pp.ni] == 1 {
                if pp.d < 0.01 {
                    c = (255, 0, 0)
                }
            }

            im.set_px(i, j, c);
        }
    }
    im.dump_to_file("level.png");
    let l = Level::default();
    let max_dist = 0.1;
    let mut im = ImageBuffer::new(l.dw, l.dh);
    for i in 0..im.w {
        for j in 0..im.h {
            let x = i as f32 / im.w as f32;
            let y = j as f32 / im.h as f32;
            let pp = l.point(x, y);
            let d = l.wall_distance(Vec2::new(x, y));
            let c = if d == 0.0 {
                (255, 0, 0)
            } else {
                ((d/max_dist*255.0) as u8, (d/max_dist*255.0) as u8, (d/max_dist*255.0) as u8)
            };
            im.set_px(i, j, c);
        }
    }
    im.dump_to_file("level_distances.png");
}