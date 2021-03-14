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
        println!("1 - Prepopulate Database w/ Demo Data");
        println!("2 - Prepopulate Database w/ Test Data");
        println!("--------");
        println!("3 - Create Production Admin");
        println!("4 - Create Test Admin");
        println!("5 - Quit");
        println!("--------");
        println!("Choose your option (1-3): ");

        let mut response = String::new();
        stdin().read_line(&mut response).expect("Unable to read input.");

        let choice: Result<i32, ParseIntError> = response.trim().to_string().parse::<i32>();

        match choice {
            Ok(i) => {
                match i {
                    1 => prepopulate_db("demo"),
                    2 => prepopulate_db("test"),
                    3 => {
                        let _ = create_prod_admin();
                    },
                    4 => {
                        let _ = create_test_admin();
                    },
                    5 => break,
                    _ => continue,
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
