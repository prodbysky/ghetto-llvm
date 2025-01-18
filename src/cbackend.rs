use crate::ir;
use std::io::Write;

pub struct CBackend {
    program: Vec<ir::IR>,
}

impl CBackend {
    pub fn new(mut program: Vec<ir::IR>) -> Self {
        program.reverse();
        Self { program }
    }

    pub fn compile(mut self) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![];
        {
            let mut file = std::io::BufWriter::new(&mut buffer);

            file.write_all(b"#include <stdlib.h>\n")?;
            file.write_all(b"#include <stdint.h>\n")?;
            file.write_all(b"#define u64 uint64_t\n")?;
            file.write_all(b"int main() {\n")?;
            while let Some(ir) = self.eat() {
                match ir {
                    ir::IR::DefineVariable { name, t, value } => {
                        file.write_all(
                            format!("{} {} = {};\n", t, name, value).as_str().as_bytes(),
                        )?;
                    }
                    ir::IR::Exit { value } => {
                        file.write_all(format!("exit({});\n", value).as_str().as_bytes())?;
                    }
                }
            }
            file.write_all(b"}\n")?;
            file.flush()?;
        }
        Ok(buffer)
    }

    fn eat(&mut self) -> Option<ir::IR> {
        self.program.pop()
    }
}
