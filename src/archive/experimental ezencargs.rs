// ez enc args didnt work at some point and I made this to fix it, I forgot why, if it bugs me again I will reimplement in a way that doesn't force dots in enc args since thats annoying

pub fn parse_encoding_args(args: &Arguments, rc: &Recipe) -> String {

    let input_enc_args = if args.encargs.is_some() {
        return args.encargs.clone().expect("Failed unwrapping --encargs");
    } else {
        rc.get("output", "enc args")
    };

    let mut enc_arg_presets: Recipe = Recipe::new();

    parse_recipe(
        current_exe()
            .expect("Failed getting exe path")
            .parent()
            .expect("Failed getting exe parent path")
            .parent()
            .unwrap()
            .join("encoding_presets.ini"),
        &mut enc_arg_presets,
    );

    let mut ret: Vec<String> = vec![];

    for word in input_enc_args.split(' ') {
        let word = word.replace("HEVC.", "H265.");
        let word = word.replace("AVC.", "H264.");

        let (_unused, map) = enc_arg_presets.data.get_key_value("FFMPEG ARGUMENT MACROS").unwrap();
        verb!("KEYS: {:?}", map.keys());

        ret.push(
            if map.contains_key(&word) && word.to_uppercase() == word {
                enc_arg_presets.get("FFMPEG ARGUMENT MACROS", &word)
            } else {
                word
            }
        );
    }
    verb!("{:?} -> {:?}", input_enc_args, ret.join(" "));

    ret.join(" ")
}
