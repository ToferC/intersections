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
        println!("1 - Create Test Admin");
        println!("2 - Create Production Admin");
        println!("--------");
        println!("3 - Prepopulate Database w/ Test Data");
        println!("4 - Prepopulate Database w/ Demo Data");
        println!("5 - Quit");
        println!("--------");
        println!("Choose your option (1-3): ");

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
                        let _ = create_prod_admin();
                    },
                    3 => prepopulate_db("test"),
                    4 => prepopulate_db("demo"),
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
