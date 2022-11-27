use capnpc::CompilerCommand;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=capnp/c2_agent.capnp");

    CompilerCommand::new()
        .src_prefix("capnp")
        .file("capnp/c2_agent.capnp")
        .run()
        .expect("Compiling schema");
}
