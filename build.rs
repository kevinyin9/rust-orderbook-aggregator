fn main(){
    tonic_build::compile_protos("proto/book.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    // tonic_build::configure()
    //     .out_dir("proto")
    //     .compile(&["proto/book.proto"], &["proto"]);
}