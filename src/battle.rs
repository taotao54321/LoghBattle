use anyhow::Context as _;
use arrayvec::ArrayVec;

use crate::util;

pub const ALLY_FLEET_COUNT: usize = 11;
pub const ENEMY_FLEET_COUNT: usize = 15;

type AllyFleetForces = ArrayVec<FleetForce, ALLY_FLEET_COUNT>;
type AllyFleetIsTireds = ArrayVec<bool, ALLY_FLEET_COUNT>;

type EnemyFleetForces = ArrayVec<FleetForce, ENEMY_FLEET_COUNT>;

const ATTACK_FORCE_MIN: u32 = 100;
const ATTACK_FORCE_MAX: u32 = 1600;

pub fn battle_simulate(query: &Query) -> Option<Report> {
    if !query.is_valid() {
        return None;
    }

    let ally_formation = query.ally.formation_modified();
    let enemy_formation = query.enemy.formation_modified();

    let (ally_attack, enemy_attack) = calc_attacks(query);

    let ally_damage_per_fleet = calc_damage_per_fleet(enemy_attack, query.ally.fleet_count());
    let enemy_damage_per_fleet = calc_damage_per_fleet(ally_attack, query.enemy.fleet_count());

    let ally_fleet_forces = damage_ally(&query.ally.fleet_forces, ally_damage_per_fleet);
    let (enemy_fleet_forces, enemy_guard_force) = damage_enemy(
        &query.enemy.fleet_forces,
        query.enemy.guard_force,
        enemy_damage_per_fleet,
    );

    Some(Report {
        ally: ReportAlly {
            formation: ally_formation,
            damage_per_fleet: ally_damage_per_fleet,
            fleet_forces: ally_fleet_forces,
        },
        enemy: ReportEnemy {
            formation: enemy_formation,
            damage_per_fleet: enemy_damage_per_fleet,
            fleet_forces: enemy_fleet_forces,
            guard_force: enemy_guard_force,
        },
    })
}

/// 味方と敵の攻撃力(ヤン補正済み)を求める。
fn calc_attacks(query: &Query) -> (u32, u32) {
    let ally_attack_force = query.ally.attack_force_clamped();
    let ally_formation = query.ally.formation_modified();

    let enemy_attack_force = query.enemy.attack_force_clamped();
    let enemy_formation = query.enemy.formation_modified();

    let mut ally_attack = calc_attack_raw(ally_attack_force, ally_formation, enemy_formation);
    let mut enemy_attack = calc_attack_raw(enemy_attack_force, enemy_formation, ally_formation);

    if query.enemy.has_yang && query.ally.formation != FORMATION_5 {
        ally_attack = ally_attack / 2 + 1;
        enemy_attack += 10;
    }

    (ally_attack, enemy_attack)
}

/// ヤン補正前の攻撃力を求める。
fn calc_attack_raw(attack_force: u32, formation_us: Formation, formation_them: Formation) -> u32 {
    let coef = formation_us.attack_coef(formation_them);

    attack_force / 100 * coef
}

/// 1 艦隊あたりのダメージを求める。
fn calc_damage_per_fleet(attack_them: u32, fleet_count_us: usize) -> u32 {
    let fleet_count_us = util::u32_from_usize(fleet_count_us);

    (100 * attack_them / (12 * fleet_count_us)).min(100)
}

/// 味方陣営にダメージを与えた結果を返す。
fn damage_ally(fleet_forces: &AllyFleetForces, damage_per_fleet: u32) -> AllyFleetForces {
    fleet_forces
        .iter()
        .map(|fleet_force| {
            let x = fleet_force.inner().saturating_sub(damage_per_fleet);
            let x = if x <= 8 { 0 } else { x };
            FleetForce::new(x).expect("damaged ally fleet force should be valid")
        })
        .collect()
}

/// 敵陣営にダメージを与えた結果を返す。
fn damage_enemy(
    fleet_forces: &EnemyFleetForces,
    guard_force: FleetForce,
    damage_per_fleet: u32,
) -> (EnemyFleetForces, FleetForce) {
    let fleet_forces: EnemyFleetForces = fleet_forces
        .iter()
        .map(|fleet_force| {
            let x = fleet_force.inner().saturating_sub(damage_per_fleet);
            let x = if x <= 8 { 0 } else { x };
            FleetForce::new(x).expect("damaged enemy fleet force should be valid")
        })
        .collect();

    let guard_force = {
        let x = guard_force.inner().saturating_sub(damage_per_fleet);
        let x = if x < 8 { 0 } else { x };
        FleetForce::new(x).expect("damaged enemy guard force should be valid")
    };

    (fleet_forces, guard_force)
}

#[derive(Debug, Default)]
pub struct Query {
    ally: QueryAlly,
    enemy: QueryEnemy,
}

impl Query {
    pub fn ally_fleet_force(&self, idx: usize) -> FleetForce {
        self.ally.fleet_forces[idx]
    }

    pub fn set_ally_fleet_force(&mut self, idx: usize, fleet_force: FleetForce) {
        self.ally.fleet_forces[idx] = fleet_force;
    }

    pub fn ally_fleet_is_tired(&self, idx: usize) -> bool {
        self.ally.fleet_is_tireds[idx]
    }

    pub fn set_ally_fleet_is_tired(&mut self, idx: usize, is_tired: bool) {
        self.ally.fleet_is_tireds[idx] = is_tired;
    }

    pub fn ally_formation(&self) -> Formation {
        self.ally.formation
    }

    pub fn set_ally_formation(&mut self, formation: Formation) {
        self.ally.formation = formation;
    }

    pub fn enemy_fleet_force(&self, idx: usize) -> FleetForce {
        self.enemy.fleet_forces[idx]
    }

    pub fn set_enemy_fleet_force(&mut self, idx: usize, fleet_force: FleetForce) {
        self.enemy.fleet_forces[idx] = fleet_force;
    }

    pub fn enemy_guard_force(&self) -> FleetForce {
        self.enemy.guard_force
    }

    pub fn set_enemy_guard_force(&mut self, fleet_force: FleetForce) {
        self.enemy.guard_force = fleet_force;
    }

    pub fn enemy_formation(&self) -> Formation {
        self.enemy.formation
    }

    pub fn set_enemy_formation(&mut self, formation: Formation) {
        self.enemy.formation = formation;
    }

    pub fn enemy_has_yang(&self) -> bool {
        self.enemy.has_yang
    }

    pub fn set_enemy_has_yang(&mut self, has_yang: bool) {
        self.enemy.has_yang = has_yang;
    }

    pub fn is_valid(&self) -> bool {
        self.ally.is_valid() && self.enemy.is_valid()
    }
}

#[derive(Debug)]
struct QueryAlly {
    fleet_forces: AllyFleetForces,
    fleet_is_tireds: AllyFleetIsTireds,
    formation: Formation,
}

impl QueryAlly {
    /// 健在な艦隊数を得る。
    fn fleet_count(&self) -> usize {
        self.fleet_forces.iter().filter(|e| !e.is_zero()).count()
    }

    /// 攻撃可能な総兵力 (clamp 済み) を得る。
    fn attack_force_clamped(&self) -> u32 {
        clamp_attack_force(self.attack_force())
    }

    /// フォーメーション (修正済み) を得る。
    fn formation_modified(&self) -> Formation {
        modify_formation(self.fleet_count(), self.attack_force(), self.formation)
    }

    /// 攻撃可能な総兵力 (clamp なし) を得る。
    fn attack_force(&self) -> u32 {
        (0..ALLY_FLEET_COUNT)
            .filter_map(|i| (!self.fleet_is_tireds[i]).then(|| self.fleet_forces[i].inner()))
            .sum()
    }

    fn is_valid(&self) -> bool {
        self.fleet_forces.iter().any(|e| !e.is_zero())
    }
}

impl Default for QueryAlly {
    fn default() -> Self {
        let mut fleet_forces = AllyFleetForces::from([FleetForce::zero(); ALLY_FLEET_COUNT]);
        fleet_forces[0] = FleetForce::MAX;

        let fleet_is_tireds = AllyFleetIsTireds::from([false; ALLY_FLEET_COUNT]);

        Self {
            fleet_forces,
            fleet_is_tireds,
            formation: FORMATION_1,
        }
    }
}

#[derive(Debug)]
struct QueryEnemy {
    fleet_forces: EnemyFleetForces,
    guard_force: FleetForce,
    formation: Formation,
    has_yang: bool,
}

impl QueryEnemy {
    /// 健在な艦隊数(駐留艦隊含む)を得る。
    fn fleet_count(&self) -> usize {
        let count_active = self.fleet_forces.iter().filter(|e| !e.is_zero()).count();
        let count_guard = if self.guard_force.is_zero() { 0 } else { 1 };

        count_active + count_guard
    }

    /// 攻撃可能な総兵力 (clamp 済み) を得る。
    fn attack_force_clamped(&self) -> u32 {
        clamp_attack_force(self.attack_force())
    }

    /// フォーメーション (修正済み) を得る。
    fn formation_modified(&self) -> Formation {
        modify_formation(self.fleet_count(), self.attack_force(), self.formation)
    }

    /// 攻撃可能な総兵力 (clamp なし) を得る。
    fn attack_force(&self) -> u32 {
        let force_active: u32 = self
            .fleet_forces
            .iter()
            .copied()
            .map(FleetForce::inner)
            .sum();

        force_active + self.guard_force.inner()
    }

    fn is_valid(&self) -> bool {
        !self.guard_force.is_zero() || self.fleet_forces.iter().any(|e| !e.is_zero())
    }
}

impl Default for QueryEnemy {
    fn default() -> Self {
        let fleet_forces = EnemyFleetForces::from([FleetForce::zero(); ENEMY_FLEET_COUNT]);

        let guard_force = FleetForce::MAX;

        Self {
            fleet_forces,
            guard_force,
            formation: FORMATION_1,
            has_yang: false,
        }
    }
}

/// 攻撃可能な総兵力を clamp して返す。
fn clamp_attack_force(attack_force: u32) -> u32 {
    num_traits::clamp(attack_force, ATTACK_FORCE_MIN, ATTACK_FORCE_MAX)
}

/// 損害率によりフォーメーションを修正して返す。
fn modify_formation(fleet_count: usize, attack_force: u32, formation: Formation) -> Formation {
    let numer = attack_force.min(ATTACK_FORCE_MAX);
    let denom = util::u32_from_usize(10 * fleet_count);

    if numer / denom <= 3 {
        FORMATION_0
    } else {
        formation
    }
}

#[derive(Debug)]
pub struct Report {
    ally: ReportAlly,
    enemy: ReportEnemy,
}

impl Report {
    pub fn ally_formation(&self) -> Formation {
        self.ally.formation
    }

    pub fn ally_damage_per_fleet(&self) -> u32 {
        self.ally.damage_per_fleet
    }

    pub fn ally_fleet_force(&self, idx: usize) -> FleetForce {
        self.ally.fleet_forces[idx]
    }

    pub fn enemy_formation(&self) -> Formation {
        self.enemy.formation
    }

    pub fn enemy_damage_per_fleet(&self) -> u32 {
        self.enemy.damage_per_fleet
    }

    pub fn enemy_fleet_force(&self, idx: usize) -> FleetForce {
        self.enemy.fleet_forces[idx]
    }

    pub fn enemy_guard_force(&self) -> FleetForce {
        self.enemy.guard_force
    }
}

#[derive(Debug)]
struct ReportAlly {
    formation: Formation,
    damage_per_fleet: u32,
    fleet_forces: AllyFleetForces,
}

#[derive(Debug)]
struct ReportEnemy {
    formation: Formation,
    damage_per_fleet: u32,
    fleet_forces: EnemyFleetForces,
    guard_force: FleetForce,
}

/// 1 個艦隊内の兵力。
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FleetForce(u32);

impl FleetForce {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(100);

    pub fn new(inner: u32) -> Option<Self> {
        (Self::MIN.0..=Self::MAX.0)
            .contains(&inner)
            .then(|| Self(inner))
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub const fn inner(self) -> u32 {
        self.0
    }
}

impl std::str::FromStr for FleetForce {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: u32 = s.parse()?;

        Self::new(inner).context("force value is out of range")
    }
}

impl std::fmt::Display for FleetForce {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Formation(u8);

const FORMATION_0: Formation = Formation(0);
const FORMATION_1: Formation = Formation(1);
const FORMATION_5: Formation = Formation(5);

impl Formation {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(7);

    pub fn new(inner: u8) -> Option<Self> {
        (Self::MIN.0..=Self::MAX.0)
            .contains(&inner)
            .then(|| Self(inner))
    }

    /// 自陣営フォーメーション self, 相手陣営フォーメーション them のときのフォーメーション係数を返す。
    fn attack_coef(self, them: Self) -> u32 {
        const COUNT: usize = (Formation::MAX.0 - Formation::MIN.0 + 1) as usize;

        const TABLE: [[u32; COUNT]; COUNT] = [
            [3, 1, 1, 1, 1, 1, 1, 1],
            [5, 4, 3, 5, 4, 2, 2, 4],
            [5, 3, 2, 3, 4, 2, 1, 3],
            [5, 3, 3, 4, 4, 4, 3, 5],
            [5, 4, 2, 4, 5, 4, 5, 5],
            [5, 4, 3, 3, 3, 3, 2, 2],
            [5, 2, 3, 3, 3, 1, 1, 3],
            [5, 4, 4, 2, 5, 5, 3, 5],
        ];

        TABLE[usize::from(self.0)][usize::from(them.0)]
    }
}

impl std::str::FromStr for Formation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: u8 = s.parse()?;

        Self::new(inner).context("formation value is out of range")
    }
}

impl std::fmt::Display for Formation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
