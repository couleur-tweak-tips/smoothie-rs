pub fn _parse_bool(boolstr: &str) -> bool {
    let pos = vec!["yes", "ye", "y", "on", "enabled", "1"];
    let neg = vec!["no", "na", "n", "off", "disabled", "0"];

    match boolstr {
        _ if pos.contains(&boolstr) => true,
        _ if neg.contains(&boolstr) => false,
        _ => panic!("Unknown boolean (true/false value): {:?}", boolstr),
    }
}

// if yes.contains(&boolstr){
//     return true;
// }else if no.contains(&boolstr) {
//     return false;
// }else {
//     panic!("Unknown boolean (true/false value): {:?}", boolstr);
// }
