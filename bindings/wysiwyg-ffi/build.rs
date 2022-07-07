fn main() {
    uniffi_build::generate_scaffolding("./src/wysiwyg_composer.udl")
        .expect("Building the UDL file failed");
}
