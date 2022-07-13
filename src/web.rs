use seed::{prelude::*, *};

use crate::battle::*;

const CLASS_HEADER_ALLY: &str = "header-ally";
const CLASS_HEADER_ENEMY: &str = "header-enemy";
const CLASS_INPUT_FLEET_FORCE: &str = "input-fleet-force";
const CLASS_INPUT_FLEET_FORCE_DEAD: &str = "input-fleet-force-dead";
const CLASS_INPUT_FORMATION: &str = "input-formation";
const CLASS_OUTPUT_FLEET_FORCE: &str = "output-fleet-force";
const CLASS_OUTPUT_FLEET_FORCE_DEAD: &str = "output-fleet-force-dead";

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

#[derive(Debug, Default)]
struct Model {
    query: Query,
}

#[derive(Debug)]
enum Msg {
    SetAllyFleetForce(usize, FleetForce),
    ToggleAllyFleetIsTired(usize),
    SetAllyFormation(Formation),
    SetEnemyFleetForce(usize, FleetForce),
    SetEnemyGuardForce(FleetForce),
    SetEnemyFormation(Formation),
    ToggleEnemyHasYang,
}

fn init(_url: Url, _orders: &mut impl Orders<Msg>) -> Model {
    Model::default()
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetAllyFleetForce(idx, fleet_force) => {
            model.query.set_ally_fleet_force(idx, fleet_force)
        }

        Msg::ToggleAllyFleetIsTired(idx) => {
            let value = !model.query.ally_fleet_is_tired(idx);
            model.query.set_ally_fleet_is_tired(idx, value);
        }

        Msg::SetAllyFormation(formation) => model.query.set_ally_formation(formation),

        Msg::SetEnemyFleetForce(idx, fleet_force) => {
            model.query.set_enemy_fleet_force(idx, fleet_force)
        }

        Msg::SetEnemyGuardForce(fleet_force) => model.query.set_enemy_guard_force(fleet_force),

        Msg::SetEnemyFormation(formation) => model.query.set_enemy_formation(formation),

        Msg::ToggleEnemyHasYang => {
            let value = !model.query.enemy_has_yang();
            model.query.set_enemy_has_yang(value);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![view_query(model), view_report(model)]
}

fn view_query(model: &Model) -> Node<Msg> {
    div![
        h2!["戦闘前"],
        view_query_ally(model),
        view_query_enemy(model),
    ]
}

fn view_query_ally(model: &Model) -> Node<Msg> {
    div![
        h3![C!(CLASS_HEADER_ALLY), "味方"],
        view_query_ally_formation(model),
        view_query_ally_fleets(model)
    ]
}

fn view_query_ally_formation(model: &Model) -> Node<Msg> {
    view_input_formation(
        "input-ally-formation",
        model.query.ally_formation(),
        Msg::SetAllyFormation,
    )
}

fn view_query_ally_fleets(model: &Model) -> Node<Msg> {
    let cols_header = (0..ALLY_FLEET_COUNT).map(|i| th![i + 1]);

    let cols_force = (0..ALLY_FLEET_COUNT).map(|i| {
        let value = model.query.ally_fleet_force(i);
        let on_change = move |fleet_force| Msg::SetAllyFleetForce(i, fleet_force);
        td![view_input_fleet_force(value, on_change)]
    });

    let cols_tired = (0..ALLY_FLEET_COUNT).map(|i| {
        td![input![
            attrs! {
                At::Type => "checkbox",
                At::Checked => model.query.ally_fleet_is_tired(i).as_at_value(),
            },
            ev(Ev::Change, move |_| Msg::ToggleAllyFleetIsTired(i)),
        ]]
    });

    table![
        thead![tr![th![], cols_header]],
        tbody![
            tr![th![label!["兵力"]], cols_force],
            tr![th![label!["疲労度80以上"]], cols_tired],
        ],
    ]
}

fn view_query_enemy(model: &Model) -> Node<Msg> {
    div![
        h3![C!(CLASS_HEADER_ENEMY), "敵"],
        view_query_enemy_formation(model),
        view_query_enemy_fleets(model),
        view_query_enemy_yang(model),
    ]
}

fn view_query_enemy_formation(model: &Model) -> Node<Msg> {
    view_input_formation(
        "input-enemy-formation",
        model.query.enemy_formation(),
        Msg::SetEnemyFormation,
    )
}

fn view_query_enemy_fleets(model: &Model) -> Node<Msg> {
    let col_guard_header = th!["駐留"];

    let col_guard_force = {
        let value = model.query.enemy_guard_force();
        td![view_input_fleet_force(value, Msg::SetEnemyGuardForce)]
    };

    let cols_fleet_header = (0..ENEMY_FLEET_COUNT).map(|i| th![i + 1]);

    let cols_fleet_force = (0..ENEMY_FLEET_COUNT).map(|i| {
        let value = model.query.enemy_fleet_force(i);
        let on_change = move |fleet_force| Msg::SetEnemyFleetForce(i, fleet_force);
        td![view_input_fleet_force(value, on_change)]
    });

    table![
        thead![tr![th![], col_guard_header, cols_fleet_header]],
        tbody![tr![th![label!["兵力"]], col_guard_force, cols_fleet_force]],
    ]
}

fn view_query_enemy_yang(model: &Model) -> Node<Msg> {
    const ID_INPUT: &str = "input-enemy-yang";

    p![
        label![
            attrs! {
                At::For => ID_INPUT,
            },
            "ヤン参戦: ",
        ],
        input![
            id!(ID_INPUT),
            attrs! {
                At::Type => "checkbox",
                At::Checked => model.query.enemy_has_yang().as_at_value(),
            },
            ev(Ev::Change, |_| Msg::ToggleEnemyHasYang),
        ],
    ]
}

fn view_report(model: &Model) -> Node<Msg> {
    div![h2!["戦闘結果"], view_report_body(model)]
}

fn view_report_body(model: &Model) -> Option<Node<Msg>> {
    battle_simulate(&model.query)
        .map(|report| div![view_report_ally(&report), view_report_enemy(&report)])
}

fn view_report_ally(report: &Report) -> Node<Msg> {
    div![
        h3![C!(CLASS_HEADER_ALLY), "味方"],
        view_output_formation(report.ally_formation()),
        view_output_damage_per_fleet(report.ally_damage_per_fleet()),
        view_report_ally_fleets(report),
    ]
}

fn view_report_ally_fleets(report: &Report) -> Node<Msg> {
    let cols_header = (0..ALLY_FLEET_COUNT).map(|i| th![i + 1]);

    let cols_force = (0..ALLY_FLEET_COUNT).map(|i| {
        let value = report.ally_fleet_force(i);
        td![view_output_fleet_force(value)]
    });

    table![
        thead![tr![th![], cols_header]],
        tbody![tr![th!["兵力"], cols_force]],
    ]
}

fn view_report_enemy(report: &Report) -> Node<Msg> {
    div![
        h3![C!(CLASS_HEADER_ENEMY), "敵"],
        view_output_formation(report.enemy_formation()),
        view_output_damage_per_fleet(report.enemy_damage_per_fleet()),
        view_report_enemy_fleets(report),
    ]
}

fn view_report_enemy_fleets(report: &Report) -> Node<Msg> {
    let col_guard_header = th!["駐留"];

    let col_guard_force = {
        let value = report.enemy_guard_force();
        td![view_output_fleet_force(value)]
    };

    let cols_fleet_header = (0..ENEMY_FLEET_COUNT).map(|i| th![i + 1]);

    let cols_fleet_force = (0..ENEMY_FLEET_COUNT).map(|i| {
        let value = report.enemy_fleet_force(i);
        td![view_output_fleet_force(value)]
    });

    table![
        thead![tr![th![], col_guard_header, cols_fleet_header]],
        tbody![tr![th!["兵力"], col_guard_force, cols_fleet_force]],
    ]
}

fn view_input_formation<F>(id: &str, value: Formation, on_change: F) -> Node<Msg>
where
    F: FnOnce(Formation) -> Msg + Clone + 'static,
{
    p![
        label![
            attrs! {
                At::For => id,
            },
            "フォーメーション: ",
        ],
        input![
            id!(id),
            C!(CLASS_INPUT_FORMATION),
            attrs! {
                At::Type => "number",
                At::Min => Formation::MIN,
                At::Max => Formation::MAX,
                At::Value => value,
            },
            input_ev(Ev::Change, |s| s.parse::<Formation>().ok().map(on_change)),
        ],
    ]
}

fn view_input_fleet_force<F>(value: FleetForce, on_change: F) -> Node<Msg>
where
    F: FnOnce(FleetForce) -> Msg + Clone + 'static,
{
    input![
        C!(
            CLASS_INPUT_FLEET_FORCE,
            IF!(value.is_zero() => CLASS_INPUT_FLEET_FORCE_DEAD),
        ),
        attrs! {
            At::Type => "number",
            At::Min => FleetForce::MIN,
            At::Max => FleetForce::MAX,
            At::Value => value,
        },
        input_ev(Ev::Change, |s| s.parse::<FleetForce>().ok().map(on_change)),
    ]
}

fn view_output_formation(value: Formation) -> Node<Msg> {
    p![format!("修正後フォーメーション: {value}")]
}

fn view_output_damage_per_fleet(value: u32) -> Node<Msg> {
    p![format!("1 個艦隊あたりのダメージ: {value}")]
}

fn view_output_fleet_force(value: FleetForce) -> Node<Msg> {
    output![
        C!(
            CLASS_OUTPUT_FLEET_FORCE,
            IF!(value.is_zero() => CLASS_OUTPUT_FLEET_FORCE_DEAD),
        ),
        value.to_string(),
    ]
}
