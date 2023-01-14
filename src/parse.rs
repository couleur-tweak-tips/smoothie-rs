pub fn parse_bool (boolean: &str) -> bool {
    let yes = vec!["yes","y"];
    let no = vec!["no","n"];
    
    if yes.contains(&boolean){
        return true;
    }else if no.contains(&boolean) {
        return false;
    }else {
        panic!("Unknown boolean (true/false value): {:?}", boolean);
    }
}
