use std::io::stdin;
use webapp::models::{UserData, User};
use database;

fn main() {

    dotenv::dotenv().ok();
    database::init();

    println!("Create superuser for intersections");

    let mut user_name: String = "".to_string();
    let mut email: String = "".to_string();
    let mut hash: String = "".to_string();

    println!("Enter Username: ");
    stdin().read_line(&mut user_name).expect("Unable to read user_name");

    println!("Enter Email: ");
    stdin().read_line(&mut email).expect("Unable to read user_name");

    println!("Enter Password (minimum 12 character): ");
    stdin().read_line(&mut hash).expect("Unable to read user_name");

    let user_data: UserData = UserData {
        user_name: user_name.to_lowercase().trim().to_string(),
        email: email.to_lowercase().trim().to_string(),
        password: hash.trim().to_string(),
        role: "admin".to_owned(),
    };

    let user = User::create(user_data).expect("Unable to create new superuser.");

    println!("New user created: {:?}", &user);

    println!("End Script")



}
