pub static ADJECTIVES: [&str; 6] = ["Mushy", "Starry", "Peaceful", "Phony", "Amazing", "Queasy"];

pub static ANIMALS: [&str; 6] = ["Owl", "Mantis", "Gopher", "Robin", "Vulture", "Prawn"];

pub fn random_name() -> String {
    let adjective = fastrand::choice(ADJECTIVES).unwrap();
    let animal = fastrand::choice(ANIMALS).unwrap();
    format!("{adjective}{animal}")
}

mod test {
    use crate::common::utils::random_name;

    #[test]
    fn test_random_name() {
        for i in 0..100 {
            println!("i = {},name = {}", i, random_name());
        }
    }
}
