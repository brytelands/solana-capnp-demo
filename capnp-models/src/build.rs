use capnpc::CompilerCommand;

fn main() {
    CompilerCommand::new()
        .file("./src/person.capnp")
        .output_path("./")
        .run()
        .expect("compiling schema");

    CompilerCommand::new()
        .file("./src/schema.capnp")
        .output_path("./")
        .run()
        .expect("compiling schema");
}