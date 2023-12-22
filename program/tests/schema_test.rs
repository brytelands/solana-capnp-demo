mod person_capnp;
mod schema_capnp;

#[cfg(test)]
mod tests {
    use capnp::message::ReaderOptions;
    use capnp::serialize;
    use capnpc::CompilerCommand;
    use solana_program::msg;

    use bryte_descriptor_client::{get_account_schema, get_raw_account_data};

    use crate::person_capnp;
    use crate::schema_capnp;

    #[tokio::test]
    pub async fn capnp_binary_schema_test() {
        // TODO update the account pubkey with the key that was generated during the capnp_demo_client run.
        // TODO update the program ID with the ID generated during deploy
        let (discriminator, data) = get_raw_account_data(
            "ATpykMvDAu4eG5YLckTLYYqzSKgmn7HBdfPvLALGYaQS".to_string(),
            "EmcSKVqz9rAwFPWDJ6YxvBLNaFjESNMrJkaREVs7MC72".to_string(),
            "http://localhost:8899".to_string(),
        )
        .await
        .unwrap();
        let schema = get_account_schema(
            "ATpykMvDAu4eG5YLckTLYYqzSKgmn7HBdfPvLALGYaQS".to_string(),
            "EmcSKVqz9rAwFPWDJ6YxvBLNaFjESNMrJkaREVs7MC72".to_string(),
            true,
            "http://localhost:8899".to_string(),
        )
        .await
        .unwrap();
        std::fs::write("./tests/person.bin", schema).unwrap();
        let result = std::fs::read("./tests/person.bin").unwrap();
        let reader = serialize::read_message(result.as_slice(), ReaderOptions::new()).unwrap();

        let schema = reader
            .get_root::<schema_capnp::code_generator_request::Reader>()
            .unwrap();
        println!("{:?}", schema);
    }

    #[tokio::test]
    pub async fn capnp_binary_schema_generation_test() {
        // TODO update the account pubkey with the key that was generated during the capnp_demo_client run.
        // TODO update the program ID with the ID generated during deploy
        // Get the discriminator and account data from your primary account.
        let (discriminator, data) = get_raw_account_data(
            "ATpykMvDAu4eG5YLckTLYYqzSKgmn7HBdfPvLALGYaQS".to_string(),
            "EmcSKVqz9rAwFPWDJ6YxvBLNaFjESNMrJkaREVs7MC72".to_string(),
            "http://localhost:8899".to_string(),
        )
        .await
        .unwrap();

        // Get the schema for your primary account. Pass the primary account pubkey and the client library will look up the account descriptor and retrieve the schema.
        let schema = get_account_schema(
            "ATpykMvDAu4eG5YLckTLYYqzSKgmn7HBdfPvLALGYaQS".to_string(),
            "EmcSKVqz9rAwFPWDJ6YxvBLNaFjESNMrJkaREVs7MC72".to_string(),
            true,
            "http://localhost:8899".to_string(),
        )
        .await
        .unwrap();
        // Save the schema to disk
        std::fs::write("./tests/person.capnp", schema).unwrap();

        // Generate Rust code for the schema
        CompilerCommand::new()
            .file("./tests/person.capnp")
            .output_path("./")
            .run()
            .expect("compiling schema");

        // Utilize the generated code to deserialize the account data.
        let reader = serialize::read_message_from_flat_slice_no_alloc(
            &mut data.as_slice(),
            ReaderOptions::new(),
        )
        .unwrap();

        let person = reader.get_root::<person_capnp::person::Reader>().unwrap();
        msg!("{:?} {:?}", person.get_firstname(), person.get_lastname());
    }
}
