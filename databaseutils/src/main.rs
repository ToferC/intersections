use std::{io::{stdin}, num::ParseIntError};

use databaseutils::{create_prod_admin, create_test_admin, prepopulate_db};
use database;

fn main() {
    
    dotenv::dotenv().ok();
    database::init();

    loop {

        println!("Welcome to Intersections ADMIN CLI interface");
        println!("--------");
        println!("OPTIONS");
        println!("1 - Prepopulate Database");
        println!("2 - Create Production Admin");
        println!("3 - Create Test Admin");
        println!("4 - Quit");
        println!("--------");
        println!("Choose your option (1-3): ");

        let mut response = String::new();
        stdin().read_line(&mut response).expect("Unable to read input.");

        let choice: Result<i32, ParseIntError> = response.trim().to_string().parse::<i32>();

        match choice {
            Ok(i) => {
                match i {
                    1 => prepopulate_db(),
                    2 => {
                        let _ = create_prod_admin();
                    },
                    3 => {
                        let _ = create_test_admin();
                    },
                    4 => break,
                    _ => continue,
                };
            },
            Err(e) => {
                println!("{}", e);
                continue
            }
        }
    };
}
