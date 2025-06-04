fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &[
            // "proto/common/date.proto",
            "proto/property_management.proto",
            "proto/user_management.proto",
            "proto/payment_management.proto",
            "proto/artifact_management.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
