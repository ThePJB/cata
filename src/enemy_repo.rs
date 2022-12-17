use crate::kmath::*;

#[derive(Clone)]
pub struct PackRecord {
    vec: Vec<(usize, usize)>,
}


#[derive(Clone, Copy)]
pub struct EnemyRecord {
    pub radius: f32,
    pub initial_hp: f32,
    pub melee_damage: f32,

    pub acquisition_radius: f32,
    pub speed_to_target: f32,
    pub speed_wander: f32,

    pub is_projectile: bool,
    pub shoot_range: f32,
    pub projectile: i32,
    pub projectile_cooldown: f32,

    pub colour_inner: Vec4,
    pub colour_outer: Vec4,
}

impl Default for EnemyRecord {
    fn default() -> Self {
        EnemyRecord {
            radius: 0.003,
            initial_hp: 0.8,
            melee_damage: 0.3,

            acquisition_radius: 0.15,
            speed_to_target: 0.03,
            speed_wander: 0.00,

            is_projectile: false,
            shoot_range: 0.0,
            projectile: -1,
            projectile_cooldown: 1.0,

            colour_inner: Vec4::new(1.0, 1.0, 1.0, 1.0),
            colour_outer: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

pub struct EnemyRepo {
    pub enemies: Vec<EnemyRecord>,
    pub spawn_table: Vec<Vec<(usize, usize)>>,
}

impl EnemyRepo {
    fn push(&mut self, er: EnemyRecord) -> usize {
        self.enemies.push(er);
        self.enemies.len() - 1
    }
    pub fn get(&self, id: usize) -> EnemyRecord {
        self.enemies[id]
    }
}

impl Default for EnemyRepo {
    fn default() -> Self {
        let mut repo = EnemyRepo { 
            enemies: Vec::new(),
            spawn_table: Vec::new(),
        };
        let mut default = EnemyRecord::default();

        let mut basic = default;
        basic.speed_to_target = 0.02;
        basic.speed_wander = 0.04;
        basic.colour_inner = Vec4::new(0.5, 0.1, 0.1, 1.0);
        let basic_id = repo.push(basic);

        let mut fast_green_projectile = default;
        fast_green_projectile.is_projectile = true;
        fast_green_projectile.speed_to_target = 0.1;
        fast_green_projectile.colour_inner = Vec4::new(0.2, 0.5, 0.0, 1.0);
        fast_green_projectile.colour_outer = Vec4::new(1., 1., 1., 1.);
        fast_green_projectile.initial_hp = 0.1;
        let fast_green_projectile_id = repo.push(fast_green_projectile);
        
        let mut slow_green_projectile = fast_green_projectile;
        slow_green_projectile.speed_to_target = 0.06;
        let slow_green_projectile_id = repo.push(slow_green_projectile);
        
        let mut shooter = default;
        shooter.speed_to_target = 0.04;
        shooter.projectile = fast_green_projectile_id as i32;
        shooter.shoot_range = 0.1;
        shooter.projectile_cooldown = 1.0;
        shooter.colour_inner = Vec4::new(0.2, 0.5, 0.0, 1.0);
        shooter.initial_hp = 0.4;
        let shooter_id = repo.push(shooter);

        let mut stationary_shooter = shooter;
        stationary_shooter.radius = 0.006;
        stationary_shooter.projectile = slow_green_projectile_id as i32;
        stationary_shooter.projectile_cooldown = 0.333;
        stationary_shooter.speed_to_target = 0.0;
        stationary_shooter.initial_hp = 2.0;
        stationary_shooter.shoot_range = 0.15;
        repo.push(stationary_shooter);
        let stationary_shooter_id = repo.push(stationary_shooter);


        let locust_colour = Vec4::new(0.7, 0.0, 0.7, 1.0);
        let mut locust_projectile = slow_green_projectile;
        locust_projectile.colour_inner = locust_colour;
        let locust_projectile_id = repo.push(locust_projectile);

        let mut locust = shooter;
        locust.initial_hp = 0.15;
        locust.projectile = locust_projectile_id as i32;
        locust.speed_to_target = 0.03;
        locust.colour_inner = locust_colour;
        locust.radius = 0.0025;
        locust.projectile_cooldown = 1.3;
        locust.acquisition_radius = 0.2;
        let locust_id = repo.push(locust);

        let mut swarm_host = locust;
        swarm_host.initial_hp = 2.0;
        swarm_host.radius = 0.006;
        swarm_host.speed_to_target = 0.01;
        swarm_host.projectile = locust_id as i32;
        swarm_host.shoot_range = 0.2;
        let swarm_host_id = repo.push(swarm_host);

        let mut rusher = default;
        rusher.initial_hp = 0.2;
        rusher.radius = 0.0025;
        rusher.speed_to_target = 0.07;
        rusher.melee_damage = 0.3;
        rusher.colour_inner = Vec4::new(0.5, 0.0, 0.0, 1.0);
        let rusher_id = repo.push(rusher);

        let mut easy_guy = basic;
        easy_guy.initial_hp = 0.5;
        easy_guy.colour_inner = Vec4::new(0.4, 0.0, 0.4, 1.0);
        easy_guy.speed_wander = 0.01;
        let easy_guy_id = repo.push(easy_guy);

        let mut easy_bullet = slow_green_projectile;
        easy_bullet.colour_inner = Vec4::new(0.4, 0.1, 0.1, 1.0);
        easy_bullet.speed_to_target = 0.04;
        let easy_bullet_id = repo.push(easy_bullet);

        let mut easy_shooter = easy_guy;
        easy_shooter.colour_inner.x *= 0.7;
        easy_shooter.colour_inner.y *= 0.7;
        easy_shooter.colour_inner.z *= 0.7;
        easy_shooter.projectile = easy_bullet_id as i32;
        easy_shooter.speed_to_target = -0.02;
        easy_shooter.shoot_range = 0.1;
        easy_shooter.acquisition_radius = 0.03;
        easy_shooter.projectile_cooldown = 2.5;
        let easy_shooter_id = repo.push(easy_shooter);

        let easy_pack = vec![(easy_guy_id, 6)];
        let easy_shooter_pack = vec![(easy_shooter_id, 6)];
        let easy_mixed_pack = vec![(easy_shooter_id, 3), (easy_guy_id, 3)];
        let easy_big_pack = vec![(easy_shooter_id, 6), (easy_guy_id, 6)];
        
        repo.spawn_table.push(easy_pack);
        repo.spawn_table.push(easy_shooter_pack);
        repo.spawn_table.push(easy_big_pack);
        repo.spawn_table.push(easy_mixed_pack);

        repo
    }
}

// const NUM_ENEMY_TYPES: usize = 11;

// const ENEMY_START_HP: [f32; NUM_ENEMY_TYPES] = [0.8, 0.2, 0.5, 999.0, 2.0, 2.0, 0.1, 999.0, 0.8, 0.5, 999.0];
// const ENEMY_ACQUISITION_RADIUS: [f32; NUM_ENEMY_TYPES] = [0.15, 0.15, 0.15, 0.15, 0.00, 0.05, 0.15, 0.00, 0.15, 0.15, 0.15];
// const ENEMY_SHOOT_RANGE: [f32; NUM_ENEMY_TYPES] = [0.0, 0.0, 0.1, 0.0, 0.1, 0.2, 0.1, 0.0, 0.1, 0.1, 0.1];
// const ENEMY_RADIUS: [f32; NUM_ENEMY_TYPES] = [0.004, 0.0025, 0.003, 0.003, 0.006, 0.005, 0.002, 0.0025, 0.003, 0.003, 0.003];
// const ENEMY_MELEE_DAMAGE: [f32; NUM_ENEMY_TYPES] = [0.4, 0.3, 0.0, 0.3, 0.0, 0.0, 0.3, 0.3, 0.3, 0.3, 0.3];
// const ENEMY_SPEED: [f32; NUM_ENEMY_TYPES] = [0.02, 0.09, 0.04, 0.12, 0.02, 0.03, 0.03, 0.08, 0.05, 0.05, 0.05];
// const ENEMY_IS_PROJECTILE: [bool; NUM_ENEMY_TYPES] = [false, false, false, true, false, false, false, true, false, true, true];
// const ENEMY_PROJECTILE: [i32; NUM_ENEMY_TYPES] = [-1, -1, 3, -1, 3, 6, 7, -1, 9, 10, -1];
// const ENEMY_PROJECTILE_COOLDOWN: [f32; NUM_ENEMY_TYPES] = [1.0, 1.0, 1.0, 1.0, 0.333, 0.7, 1.0, 1.0, 0.666, 0.666, 0.666];

// const ENEMY_COLOUR_INNER: [Vec4; NUM_ENEMY_TYPES] = [
//     Vec4::new(0.5, 0.1, 0.1, 1.0),
//     Vec4::new(0.5, 0.0, 0.0, 1.0),
//     Vec4::new(0.2, 0.7, 0.0, 1.0),
//     Vec4::new(0.2, 0.5, 0.0, 1.0),
//     Vec4::new(0.2, 0.5, 0.0, 1.0),
//     Vec4::new(0.2, 0.0, 0.2, 1.0),
//     Vec4::new(0.25, 0.0, 0.25, 1.0),
//     Vec4::new(0.5, 0.0, 0.5, 1.0),
//     Vec4::new(0.7, 0.7, 0.0, 1.0),
//     Vec4::new(0.7, 0.7, 0.0, 1.0),
//     Vec4::new(0.7, 0.7, 0.0, 1.0),
// ];
// const ENEMY_COLOUR_OUTER: [Vec4; NUM_ENEMY_TYPES] = [
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(1.0, 1.0, 1.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(1.0, 1.0, 1.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(0.0, 0.0, 0.0, 1.0),
//     Vec4::new(1.0, 1.0, 1.0, 1.0),
// ];
