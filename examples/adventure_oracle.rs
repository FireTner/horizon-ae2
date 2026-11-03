use ae2_horizon::ae2_adventure::{self as rules, Domain, Kind, Registry, Value, World};
use serde_json::{Value as Json, json};
use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};
#[derive(Clone, Debug, PartialEq)]
enum Data {
    Empty,
    Signed(i64),
    Bits(u64),
    Text(Vec<u16>),
    List(Vec<Node>),
    Map(BTreeMap<Vec<u16>, Node>),
}
#[derive(Clone, Debug, PartialEq)]
struct Node {
    kind: u8,
    data: Data,
}
impl Node {
    fn read(v: &Json) -> Self {
        let kind = v[0].as_u64().unwrap() as u8;
        let data = match kind {
            1..=4 => Data::Signed(v[1].as_i64().unwrap()),
            5 | 6 => Data::Bits(v[1].as_str().unwrap().parse().unwrap()),
            8 => Data::Text(
                v[1].as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u16)
                    .collect(),
            ),
            7 | 9 | 11 | 12 => {
                Data::List(v[1].as_array().unwrap().iter().map(Self::read).collect())
            }
            10 => Data::Map(
                v[1].as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        (
                            v[0].as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u16)
                                .collect(),
                            Self::read(&v[1]),
                        )
                    })
                    .collect(),
            ),
            _ => Data::Empty,
        };
        Self { kind, data }
    }
}
impl Node {
    fn view(&self) -> Json {
        let data = match &self.data {
            Data::Empty => Json::Null,
            Data::Signed(n) => json!(n),
            Data::Bits(n) => json!(n.to_string()),
            Data::Text(v) => json!(v),
            Data::List(v) => json!(v.iter().map(Self::view).collect::<Vec<_>>()),
            Data::Map(v) => json!(
                v.iter()
                    .map(|(k, v)| json!([k, v.view()]))
                    .collect::<Vec<_>>()
            ),
        };
        json!([self.kind, data])
    }
}
impl ae2_horizon::ae2_adventure_snbt::Target for Node {
    fn number(kind: u8, bits: u64) -> Self {
        Self {
            kind,
            data: if kind >= 5 {
                Data::Bits(bits)
            } else {
                Data::Signed(bits as i64)
            },
        }
    }
    fn string(v: Vec<u16>) -> Self {
        Self {
            kind: 8,
            data: Data::Text(v),
        }
    }
    fn compound(v: Vec<(Vec<u16>, Self)>) -> Self {
        Self {
            kind: 10,
            data: Data::Map(v.into_iter().collect()),
        }
    }
    fn sequence(kind: u8, v: Vec<Self>) -> Self {
        Self {
            kind,
            data: Data::List(v),
        }
    }
}
impl Value for Node {
    type Key = Vec<u16>;
    fn kind(&self) -> u8 {
        self.kind
    }
    fn text(&self) -> Option<String> {
        if let Data::Text(s) = &self.data {
            Some(String::from_utf16_lossy(s))
        } else {
            None
        }
    }
    fn boolean(&self) -> Option<bool> {
        match self.data {
            Data::Signed(n) => Some(n as i8 != 0),
            Data::Bits(n) if self.kind == 5 => Some(f32::from_bits(n as u32) as i32 as i8 != 0),
            Data::Bits(n) => Some(f64::from_bits(n) as i32 as i8 != 0),
            _ => None,
        }
    }
    fn field(&self, name: &str) -> Option<&Self> {
        self.entry(&name.encode_utf16().collect())
    }
    fn entry(&self, key: &Vec<u16>) -> Option<&Self> {
        if let Data::Map(m) = &self.data {
            m.get(key)
        } else {
            None
        }
    }
    fn entries(&self) -> Option<Vec<(&Vec<u16>, &Self)>> {
        if let Data::Map(m) = &self.data {
            Some(m.iter().collect())
        } else {
            None
        }
    }
    fn key_text(k: &Vec<u16>) -> String {
        String::from_utf16_lossy(k)
    }
    fn stream(&self) -> Option<Vec<Self>> {
        if let Data::List(v) = &self.data {
            Some(v.clone())
        } else {
            None
        }
    }
    fn list(&self) -> Option<&[Self]> {
        if self.kind == 9
            && let Data::List(v) = &self.data
        {
            return Some(v);
        }
        None
    }
    fn scalar_equal(&self, other: &Self) -> bool {
        if self.kind != other.kind {
            return false;
        }
        match (&self.data, &other.data) {
            (Data::Bits(a), Data::Bits(b)) if self.kind == 5 => {
                f32::from_bits(*a as u32) == f32::from_bits(*b as u32)
            }
            (Data::Bits(a), Data::Bits(b)) => f64::from_bits(*a) == f64::from_bits(*b),
            _ => self == other,
        }
    }
}
struct Environment<'a> {
    properties: &'a Json,
    case: &'a Json,
    row: &'a Json,
}
impl Registry<Node> for Environment<'_> {
    fn block_exists(&self, id: &str) -> bool {
        self.properties.get(id).is_some()
    }
    fn tag_exists(&self, id: &str) -> bool {
        self.case["tags"].get(id).is_some()
    }
    fn compound(&self, v: &Node) -> Option<Node> {
        if v.kind == 10 {
            Some(v.clone())
        } else if let Data::Text(text) = &v.data {
            ae2_horizon::ae2_adventure_snbt::parse(text)
        } else {
            None
        }
    }
}
impl Environment<'_> {
    fn domain(&self, name: &str) -> Option<Domain> {
        let d = self
            .properties
            .get(self.case["block"].as_str()?)?
            .get(name)?;
        Some(Domain {
            kind: match d["kind"].as_str()? {
                "integer" => Kind::Integer,
                "boolean" => Kind::Boolean,
                _ => Kind::Enum,
            },
            values: d["values"]
                .as_object()?
                .iter()
                .map(|(k, v)| (k.clone(), v.as_i64().unwrap() as i32))
                .collect(),
        })
    }
}
impl World<Node> for Environment<'_> {
    fn block(&self) -> Option<&str> {
        self.case["block"].as_str()
    }
    fn current_rank(&self, name: &str) -> Option<i32> {
        self.domain(name)?
            .rank(self.row["state_view"].get(name)?.as_str()?)
    }
    fn property_rank(&self, name: &str, value: &str) -> Option<i32> {
        self.domain(name)?.rank(value)
    }
    fn in_tag(&self, id: &str) -> bool {
        self.case["tags"][id]
            .as_array()
            .is_some_and(|v| v.iter().any(|n| n.as_str() == self.block()))
    }
    fn entity(&self) -> Option<Node> {
        self.row.get("entity_view").map(Node::read)
    }
}
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let properties: Json = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for line in io::stdin().lock().lines() {
        let row: Json = serde_json::from_str(&line.unwrap()).unwrap();
        if let Some(value) = row.get("integer") {
            let text = if let Some(units) = value.as_array() {
                String::from_utf16_lossy(
                    &units
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u16)
                        .collect::<Vec<_>>(),
                )
            } else {
                value.as_str().unwrap().to_owned()
            };
            println!("{}", json!({"integer":rules::java_integer(&text)}));
            continue;
        }
        if let Some(value) = row.get("parse_snbt") {
            let units = value.as_str().unwrap().encode_utf16().collect::<Vec<_>>();
            let result = ae2_horizon::ae2_adventure_snbt::parse::<Node>(&units).map(|n| n.view());
            println!("{}", json!({"parse_snbt":result}));
            continue;
        }
        if row.get("expected").is_some() {
            println!(
                "{}",
                json!({"compare_nbt":rules::compare_nbt(&Node::read(&row["expected"]),&Node::read(&row["actual"]))})
            );
            continue;
        }
        let env = Environment {
            properties: &properties,
            case: &row["input"],
            row: &row,
        };
        let value = Node::read(&row["component_view"]);
        let result = match rules::decode(&value, &env) {
            Ok(p) => {
                json!({"decoded":true,"show":p.show_in_tooltip,"matches":p.matches_fresh(&env)})
            }
            Err(_) => json!({"decoded":false}),
        };
        println!("{result}");
    }
}
