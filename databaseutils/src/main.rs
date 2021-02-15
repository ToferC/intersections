use std::io::stdin;
use database;
use webapp;

fn main() {
    println!("Create superuser for intersections");

    let mut user_name: String;
    let mut email: String;
    let mut hash: String;

    println!("Enter Username: ");
    stdin().read_line(&mut user_name).expect("Unable to read user_name");

    println!("Enter Email: ");
    stdin().read_line(&mut email).expect("Unable to read user_name");

    println!("Enter Password: ");
    stdin().read_line(&mut hash).expect("Unable to read user_name");

    let user = webapp::models::User::new();

}
