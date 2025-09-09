#[cfg(feature = "tokens")]
mod tokens {
    use std::{
        env,
        fs::{read_to_string, File},
        io::{BufWriter, Write},
        path::Path,
    };

    use phf_codegen::Map;

    /// Codegener for the token data
    /// This will generate a static map of tokens to strings at build time
    /// This is used to resolve tokens in the game data on runtime.
    pub fn create_token_file<P: AsRef<Path>>(
        token_filename: P,
        output_filename: &'static str,
        variable_name: &'static str,
    ) {
        let path = Path::new(&env::var("OUT_DIR").unwrap()).join(output_filename);
        let mut file = BufWriter::new(File::create(&path).unwrap());

        let contents = read_to_string(token_filename).unwrap();
        let mut map = Map::new();
        for line in contents.lines() {
            let mut parts = line.splitn(2, ' ');
            let value = parts.next().unwrap();
            let token = parts.next().unwrap().parse::<u16>().unwrap();
            map.entry(token, format!("\"{}\"", value));
        }
        write!(
            &mut file,
            "const {}: phf::Map<u16, &'static str> = {}",
            variable_name,
            map.build()
        )
        .unwrap();
        write!(&mut file, ";\n").unwrap();
    }
}

fn main() {
    #[cfg(feature = "tokens")]
    {
        use std::{env, path::Path};

        const TOKENS_FILE: &str = "tokens_1.tok";
        let tokens_path =
            Path::new(&env::var("TOKENS_DIR").unwrap_or(".".to_owned())).join(TOKENS_FILE);

        tokens::create_token_file(tokens_path, "token_data.rs", "TOKENS");
    }
}
