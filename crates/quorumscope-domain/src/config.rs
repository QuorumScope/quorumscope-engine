pub trait AppConfig: Sized {
    fn from_env() -> Result<Self, String>;
}
