use crate::machine::{Datum, Machine};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DeParameter {
    pub name: Option<String>,
    pub register: Option<String>,
    pub address: Option<u16>,
    pub cost: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct DeTest {
    pub steps: Vec<DeStep>,
}

#[derive(Deserialize, Debug)]
pub struct DeStep {
    pub datum: String,
    pub set: Option<i32>,
    pub dontcare: Option<i32>,
    pub ham: Option<i32>,
    pub diff: Option<i32>,
    pub check: Option<String>,
}

#[derive(Clone, Copy)]
pub enum Step {
    Set(Datum, i32),
    Run,
    Ham(Datum, i32, i32),
    Diff(Datum, i32),
    NonZero(Datum),
    Positive(Datum),
    Negative(Datum),
}

pub struct Test {
    pub steps: Vec<Step>,
}

#[derive(Deserialize, Debug)]
pub struct DeTestRun {
    pub tests: Vec<DeTest>,
}

pub struct TestRun {
    pub tests: Vec<Test>,
}

fn step(mach: Machine, s: &DeStep) -> Step {
    match (s.set, s.dontcare, s.ham, s.diff, s.check.as_ref()) {
        (val, None, None, None, None) => Step::Set(mach.register_by_name(&s.datum), val.unwrap()),
        (None, None, val, None, None) => {
            Step::Ham(mach.register_by_name(&s.datum), val.unwrap(), -1)
        }
        (None, dc, val, None, None) => {
            Step::Ham(mach.register_by_name(&s.datum), val.unwrap(), dc.unwrap())
        }
        (None, None, None, val, None) => Step::Diff(mach.register_by_name(&s.datum), val.unwrap()),
        (None, None, None, None, Some(func)) => match func.as_str() {
            "nonzero" => Step::NonZero(mach.register_by_name(&s.datum)),
            "positive" => Step::Positive(mach.register_by_name(&s.datum)),
            "negative" => Step::Negative(mach.register_by_name(&s.datum)),
            _ => unimplemented!(),
        },
        _ => panic!(),
    }
}

fn de_step(mach: Machine, d: &[DeStep]) -> Vec<Step> {
    let st: Vec<Step> = d.iter().map(|t| step(mach, t)).collect();
    let (mut setup, mut checks): (Vec<Step>, Vec<Step>) = st.iter().partition(|s| matches!(s, Step::Set(_, _)));

    setup.push(Step::Run);
    setup.append(&mut checks);
    setup
}

fn de_test(mach: Machine, d: &DeTest) -> Test {
    Test {
        steps: de_step(mach, &d.steps),
    }
}

pub fn sanity(dtr: &DeTestRun, mach: Machine) -> TestRun {
    TestRun {
        tests: dtr.tests.iter().map(|t| de_test(mach, t)).collect(),
    }
}
