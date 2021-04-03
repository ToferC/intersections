use std::{io::{stdin}, num::ParseIntError};

use databaseutils::{create_user, create_test_admin, prepopulate_db};
use database;

fn main() {
    
    dotenv::dotenv().ok();
    database::init();

    loop {

        println!("Welcome to Intersections ADMIN CLI interface");
        println!("--------");
        println!("OPTIONS");
        println!("1 - Create Test Admin");
        println!("2 - Create Production Admin");
        println!("3 - Create User");
        println!("--------");
        println!("4 - Prepopulate Database w/ Test Data");
        println!("5 - Prepopulate Database w/ Demo Data");
        println!("6 - Quit");
        println!("--------");
        println!("Choose your option (1-6): ");

        let mut response = String::new();
        stdin().read_line(&mut response).expect("Unable to read input.");

        let choice: Result<i32, ParseIntError> = response.trim().to_string().parse::<i32>();

        match choice {
            Ok(i) => {
                match i {
                    1 => {
                        let _ = create_test_admin();
                    },
                    2 => {
                        let _ = create_user("admin");
                    },
                    3 => {
                        let _ = create_user("user");
                    }
                    4 => prepopulate_db("test"),
                    5 => prepopulate_db("demo"),
                    6 => break,
                    _ => {
                        println!("Command not recognized. Please try again.\n");
                        continue
                    },
                };
            },
            Err(e) => {
                println!("{}", e);
                continue
            }
        }

        println!("COMPLETE\n\n");
    };
}
