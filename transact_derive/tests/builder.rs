// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use transact_derive::Builder;

#[derive(Debug, PartialEq)]
pub enum BuilderError {
    MissingField(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BuilderError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

pub trait Build {
    type Result;

    fn build(self) -> Self::Result;
}

#[derive(Builder, Debug)]
#[gen_build_impl]
pub struct Agent {
    #[getter]
    public_key: String,

    #[getter]
    wears_crocks: bool,

    #[getter]
    known_enemies: Vec<String>,

    #[getter]
    #[optional]
    role: String,
}

#[derive(Builder, Debug)]
#[gen_build_impl]
#[builder_name = "OrgBuilder"]
pub struct Organization {
    #[getter]
    org_id: String,
}

#[derive(Builder)]
pub struct Payload {
    #[getter]
    action: String,

    #[getter]
    payload: Vec<u8>,
}

impl Build for PayloadBuilder {
    type Result = Result<Payload, BuilderError>;

    fn build(self) -> Self::Result {
        Ok(Payload {
            action: "EMPTY".to_string(),
            payload: Vec::new(),
        })
    }
}

#[test]
fn test_agent_builder() {
    let builder = AgentBuilder::new()
        .with_public_key("wut1234".into())
        .with_wears_crocks(false)
        .with_role("admin".into())
        .with_known_enemies(vec!["tim".to_string(), "jimmy".to_string()]);

    let agent = builder.build().unwrap();

    assert_eq!("wut1234", agent.public_key());
    assert_eq!(false, *agent.wears_crocks());
    assert_eq!(
        &["tim".to_string(), "jimmy".to_string()],
        agent.known_enemies()
    );
    assert_eq!("admin", agent.role());
}

#[test]
fn test_agent_builder_optional_field() {
    let builder = AgentBuilder::new()
        .with_public_key("wut1234".into())
        .with_wears_crocks(false)
        .with_known_enemies(vec!["tim".to_string(), "jimmy".to_string()]);

    let agent = builder.build().unwrap();

    assert_eq!("wut1234", agent.public_key());
    assert_eq!(false, *agent.wears_crocks());
    assert_eq!(
        &["tim".to_string(), "jimmy".to_string()],
        agent.known_enemies()
    );
    assert_eq!("", agent.role());
}

#[test]
fn test_agent_builder_error_on_required_field() {
    let builder = AgentBuilder::new()
        .with_wears_crocks(false)
        .with_known_enemies(vec!["tim".to_string(), "jimmy".to_string()]);

    let agent_result = builder.build();

    assert!(agent_result.is_err());

    let expected_err = BuilderError::MissingField("public_key".to_string());

    let err = agent_result.unwrap_err();

    assert_eq!(expected_err, err);
}

#[test]
fn test_custom_builder_name() {
    let org = OrgBuilder::new()
        .with_org_id("rywerx1234".into())
        .build()
        .unwrap();

    assert_eq!("rywerx1234", org.org_id());
}

#[test]
fn test_custom_build() {
    let payload = PayloadBuilder::new()
        .with_action("CreateAgent".into())
        .with_payload(Vec::new())
        .build()
        .unwrap();

    assert_eq!("EMPTY", payload.action());
}
