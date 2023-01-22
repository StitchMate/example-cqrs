use std::fmt::Debug;

pub trait TAccountServices {
    fn hash_password(&self, password: String) -> Result<String, anyhow::Error>;
}

pub trait AccountServices: TAccountServices + Debug {}