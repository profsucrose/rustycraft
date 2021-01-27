use adjective_adjective_animal::Generator;

pub fn gen_name() -> String {
    // get adjective-animal, omit first adjective
    // for length constraints
    let name = Generator::default().next().unwrap();
    let mut result = String::new();
    let mut adding_chars = false;
    for ch in name.chars().skip(1) {
        if !adding_chars && ch.is_uppercase() {
            adding_chars = true;
        }
        
        if adding_chars {
            result.push(ch);
        }
    }
    result
}