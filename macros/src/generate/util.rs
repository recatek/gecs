use convert_case::{Case, Casing};

pub fn to_snake(name: &String) -> String {
    // Remove generic argument brackets, but keep the identifiers concatenated
    let name = name.replace(&['<', '>'][..], "");
    name.from_case(Case::Pascal).to_case(Case::Snake)
}
