use serde::Deserialize;
use crate::Register;

#[derive(Deserialize, Debug)]
pub struct DeParameter {
    pub name: Option<String>,
    pub register: Option<String>,
    pub address: Option<u16>,
    pub cost: Option<f64>,
}

pub struct Parameter {
    pub name: String,
    pub address: Option<u16>,
    pub cost: Option<f64>,
    pub register: Register
}

#[derive(Deserialize, Debug)]
pub struct DeTest {
    pub ins: Vec<i8>,
    pub outs: Vec<i8>,
}

pub struct Test {
    pub ins: Vec<i8>,
    pub outs: Vec<i8>,
}

#[derive(Deserialize, Debug)]
pub struct DeTestRun {
    pub ins: Vec<DeParameter>,
    pub outs: Vec<DeParameter>,
    pub tests: Vec<DeTest>,
}

pub struct TestRun {
    pub ins: Vec<Parameter>,
    pub outs: Vec<Parameter>,
    pub tests: Vec<Test>,
}

pub fn sanity(dtr: &DeTestRun, pinj: fn(&DeParameter) -> Parameter) -> TestRun {
    TestRun {
        ins: dtr.ins.iter().map(pinj).collect(),
        outs: dtr.outs.iter().map(pinj).collect(),
        tests: dtr
            .tests
            .iter()
            .map(|d| Test {
                ins: d.ins.clone(),
                outs: d.outs.clone(),
            })
            .collect(),
    }
}
