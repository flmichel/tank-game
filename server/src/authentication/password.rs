use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string();

    password_hash
}

pub fn hash_is_correct(expected_password_hash: String, password_candidate: String) -> bool {
    let expected_password_hash = PasswordHash::new(&expected_password_hash).unwrap();

    let result =
        Argon2::default().verify_password(password_candidate.as_bytes(), &expected_password_hash);

    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}
