use chrono::DateTime;
use chrono::Utc;
use utils::generator::Generator;
use uuid::Uuid;

pub struct Generators;

impl Generator<Uuid> for Generators {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

impl Generator<DateTime<Utc>> for Generators {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
