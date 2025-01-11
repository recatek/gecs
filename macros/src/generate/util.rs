use convert_case::{Case, Casing};

pub fn to_snake(name: &String) -> String {
    name.from_case(Case::Pascal).to_case(Case::Snake)
}
