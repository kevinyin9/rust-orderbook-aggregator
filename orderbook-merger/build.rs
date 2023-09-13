fn main(){
    tonic_build::compile_protos("proto/orderbook_summary.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    // tonic_build::configure()
    //     .out_dir("proto")
    //     .compile(&["proto/orderbooksummary.proto"], &["proto"]);
}